//! Open and read .dict or .dict.dz files
//!
//! This module contains traits and structs to work with uncompressed .dict and compressed .dict.dz
//! files. These files contain the actual dictionary content. While these readers return the
//! definitions, they do not do any post-processing. Definitions are normally plain text, but they
//! could be HTML, or anything else, in theory (although plain text is the de facto default).
//!
//! To understand some of the constants defined in this module or to understand the internals of
//! the DictReaderDz struct, it is advisable to have a brief look at
//! [the GZip standard](https://tools.ietf.org/html/rfc1952).

use byteorder::{ByteOrder, LittleEndian};
use flate2;
use memmap;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufRead, Read, Seek, SeekFrom};

use errors::DictError;
use file_access::MmappedFileSlice;

/// limit size of a word buffer, so that  malicious index files cannot request to much memory for a
/// translation
pub static MAX_BYTES_FOR_BUFFER: u64 = 1048576; // no headword definition is larger than 1M

/// byte mask to query for existence of FEXTRA field in the flags byte of a `.dz` file
pub static GZ_FEXTRA: u8  = 0b00000100;
/// byte mask to query for the existence of a file name in a `.dz` file
pub static GZ_FNAME: u8   = 0b00001000; // indicates whether a file name is contained in the archive
/// byte mask to query for the existence of a comment in a `.dz` file
pub static GZ_COMMENT: u8 = 0b00010000; // indicates whether a comment is present
/// byte mask to detect that a comment is contained in a `.dz` file
pub static GZ_FHCRC: u8   = 0b00000010;


/// .dict file format: either compressed or uncompressed
/// A dictionary (content) reader
///
/// This type abstracts from the underlying seek operations required for lookup
/// of headwords and provides easy methods to search for a word given a certain
/// offset and length. Users of a type which implements this trait don't need to care about compression
/// of the dictionary.
pub trait DictReader {
    /// fetch the definition from the dictionary at offset and length
    fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError>;
}

/// Raw Dict reader
///
/// This reader can read uncompressed .dict files.
pub struct DictReaderRaw<B: Read + Seek> {
    dict_data: B,
    total_length: u64,
}

impl<B: Read + Seek> DictReaderRaw<B> {
    /// Get a new DictReader from a Reader.
    pub fn new(mut dict_data: B) -> Result<DictReaderRaw<B>, DictError> {
        let end = dict_data.seek(SeekFrom::End(0))?;
        Ok(DictReaderRaw { dict_data: dict_data, total_length: end })
    }
}

impl<B: Read + Seek> DictReader for DictReaderRaw<B> {
    /// fetch definition from dictionary
    fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError> {
        if length > MAX_BYTES_FOR_BUFFER {
            return Err(DictError::MemoryError);
        }
        if (start_offset + length) > self.total_length {
            return Err(DictError::IoError(io::Error::new(io::ErrorKind::UnexpectedEof, "a \
                      seek beyond the end of uncompressed data was requested")));
        }

        self.dict_data.seek(SeekFrom::Start(start_offset))?;
        let mut read_data = vec![0; length as usize];
        let bytes_read = try!(self.dict_data.read(read_data.as_mut_slice())) as u64;
        if bytes_read != length { // reading from end of file?
            return Err(DictError::IoError(io::Error::new(
                            io::ErrorKind::UnexpectedEof, "seek beyond end of file")));
        }
        Ok(String::from_utf8(read_data)?)
    }
}

/// Load a [DictReader](trait.DictReader.html) from file.
///
/// This function loads a [Dictreader](trait.DictReader.html) from a file and transparently selects
/// the correct reader using the file type extension, so the callee doesn't need to care about
/// compression  (`.dz`).
///
/// # Errors
///
/// The function can return a `DictError`, which can either occur if a I/O error occurs, or when
/// the GZ compressed file is invalid.
pub fn load_dict<P>(path: P) -> Result<Box<DictReader>, DictError>
		where P: AsRef<Path> {
    if path.as_ref().ends_with(".dz") {
        let dzdict = MmappedFileSlice::new(path.as_ref())?;
        Ok(Box::new(DictReaderDz::new(dzdict)?))
    } else {
        let reader = BufReader::new(File::open(path)?);
        Ok(Box::new(DictReaderRaw::new(reader)?))
    }
}


// -----------------------------------------------------------------------------
// gzip handling

/// Gzip Dict reader
///
/// This reader can read compressed .dict files. with the file name suffix .dz.
/// This format is documented in RFC 1952 and in `man dictzip`. An example implementation can be
/// found in the dict daemon (dictd) in `data.c`.
pub struct DictReaderDz {
    /// memory mapped dictionary (compressed)
    dzdict: MmappedDict,
    /// length of an uncompressed chunk
    uchunk_length: usize,
    /// end of compressed data
    end_compressed_data: usize,
    /// offsets in file where a new compressed chunk starts
    chunk_offsets: Vec<usize>,
    /// total size of uncompressed file
    ufile_length: u64, // has u64 to be quicker in comparing to offsets
}

pub struct MmappedDict {
    mmap: memmap::Mmap,
}

impl MmappedDict {
    pub fn new(path: &str) -> Result<MmappedDict, DictError> {
        let mmap = memmap::Mmap::open_path(path, memmap::Protection::Read)?;
        Ok(MmappedDict { mmap: mmap })
    }

    /// Retrieve the mapped memory region as a byte slice
    pub fn as_slice(&self) -> &[u8] {
        // Mmap::as_slice() is unsafe because the caller must guarantee that the
        // referenced memory is never concurrently modified. This is ensured
        // because new() always creates a read-only memory map. Thus, it is okay
        // to wrap this call in a safe method.
        unsafe {
            self.mmap.as_slice()
        }
    }
}

#[derive(Debug)]
// a (GZ) chunk, representing length and offset withing the compressed file
struct Chunk {
    offset: usize,
    length: usize,
}

impl DictReaderDz {
    /// Get a new DictReader from a Reader.
    pub fn new(dzdict: MmappedDict) -> Result<DictReaderDz, DictError> {
        if dzdict.as_slice()[0..2] != [0x1F, 0x8B] {
            return Err(DictError::InvalidFileFormat("Not in gzip format".into(), None));
        }
    
        let flags = dzdict.as_slice()[3]; // bitmap of gzip attributes
        if (flags & GZ_FEXTRA) == 0 { // check whether FLG.FEXTRA is set
            return Err(DictError::InvalidFileFormat("Extra flag (FLG.FEXTRA) \
                       not set, not in gzip + dzip format".into(), None));
        }
    
        // read XLEN, length of extra FEXTRA field
        let xlen = LittleEndian::read_u16(&dzdict.as_slice()[10..12]);
    
        // start of FEXTRA data (after byte 12)
        if dzdict[12..14] != ['R' as u8, 'A' as u8] {
            return Err(DictError::InvalidFileFormat("No dictzip info found in FEXTRA \
                    header (behind XLEN, in SI1SI2 fields)".into(), None));
        }
    
        let length_subfield = LittleEndian::read_u16(&dzdict.as_slice()[14..16]);
        assert_eq!(length_subfield, xlen - 4, "the length of the subfield \
                   should be the same as the fextra field, ignoring the \
                   additional length information and the file format identification");
        let subf_version = LittleEndian::read_u16(&dzdict.as_slice()[16..18]);
        if subf_version != 1 {
             return Err(DictError::InvalidFileFormat("Unimplemented dictzip \
                     version, only ver 1 supported".into(), None));
        }
    
        // before compression, the file is split into evenly-sized chunks and the size information
        // is put right after the version information:
        let uchunk_length = LittleEndian::read_u16(&dzdict.as_slice()[18..20]);
        // number of chunks in the file
        let chunk_count = LittleEndian::read_u16(&dzdict.as_slice()[20..22]);
        if chunk_count == 0 {
            return Err(DictError::InvalidFileFormat("No compressed chunks in \
                    file or broken header information".into(), None));
        }
    
        // compute number of possible chunks which would fit into the FEXTRA field; used for
        // validity check. first 10 bytes of FEXTRA are header information, the rest are 2-byte,
        // little-endian numbers.
        let numbers_chunks_which_would_fit = ((xlen - 10) / 2) as u16; // each chunk represented by u16 == 2 bytes
        // check that number of claimed chunks fits within given size for subfield
        if numbers_chunks_which_would_fit != chunk_count {
            return Err(DictError::InvalidFileFormat(format!("Expected {} chunks \
                      according to dictzip header, but the FEXTRA field can \
                      accomodate {}; possibly broken file", chunk_count,
                      numbers_chunks_which_would_fit), None));
        }
    
        let mut pos_in_buffer = 12 + xlen as usize;
        // if file name bit set, seek beyond the 0-terminated file name, we don't care
        if (flags & GZ_FNAME) != 0 {
            // fname starts _after_ xlen field
            pos_in_buffer += dzdict.as_slice().iter().skip(pos_in_buffer).position(
                |&x| x == '\0' as u8).unwrap() as usize; // todo, kein unwrap
        }
    
        // seek past comment, if any
        if (flags & GZ_COMMENT) != 0 {
            pos_in_buffer += dzdict.as_slice().iter().skip(pos_in_buffer).position(
                |&x| x == '\0' as u8).unwrap() as usize;
        }
    
        // skip CRC stuff, 2 bytes
        if (flags & GZ_FHCRC) != 0 {
            pos_in_buffer += 2;
        }
    
        // save length of each compressed chunk
        let mut chunk_offsets = Vec::with_capacity(chunk_count as usize);

        let end_of_compressed = pos_in_buffer;

        {
            // after the various header bytes parsed above, the list of chunk lengths can be found (slice for easier indexing)
            let chunks_from_header = &dzdict.as_slice()[22usize..(22 + chunk_count * 2) as usize];

            // iterate over each 2nd byte, parse u16
            for index in (0..chunks_from_header.len()).filter(|i| (i%2)==0) {
                let index = index as usize;
                let compressed_len = LittleEndian::read_u16(&chunks_from_header[index..(index + 2)]) as usize;
                chunk_offsets.push(end_of_compressed);
                pos_in_buffer += compressed_len;
            }
        }
        assert_eq!(chunk_offsets.len() as u16, chunk_count, "The read number of compressed chunks in \
                the .dz file must be equivalent to the number of chunks actually found in the file.\n");

        // read uncompressed file length, located at  EOF - 8 (or at end of compressed data)
        let uncompressed = LittleEndian::read_i32(
                &dzdict.as_slice()[end_of_compressed..end_of_compressed+4]);

        Ok(DictReaderDz { dzdict: dzdict,
                chunk_offsets: chunk_offsets,
                end_compressed_data: end_of_compressed,
                uchunk_length: uchunk_length as usize,
                ufile_length: uncompressed as u64 })
    }

    fn get_chunks_for(&self, start_offset: u64, length: u64) -> Result<Vec<Chunk>, DictError> {
        let mut chunks = Vec::new();
        let start_chunk = start_offset as usize / self.uchunk_length;
        let end_chunk = (start_offset + length) as usize / self.uchunk_length;
        for id in start_chunk..(end_chunk + 1) {
            let chunk_length = match self.chunk_offsets.get(id+1) {
                Some(next) => next - self.chunk_offsets[id],
                None => self.end_compressed_data - self.chunk_offsets[id],
            };
            chunks.push(Chunk { offset: self.chunk_offsets[id], length: chunk_length });
        }

        Ok(chunks)
    }

    // inflate a dictdz chunk
    fn inflate(&self, data: &[u8]) -> Result<Vec<u8>, DictError> {
        let mut decoder = flate2::Decompress::new(false);
        let mut decoded = vec![0u8; self.uchunk_length];
        decoder.decompress(data, decoded.as_mut_slice(), flate2::Flush::None)?;
        Ok(decoded)
    }
}

impl DictReader for DictReaderDz {
    // Fetch definition from the dictionary.
    fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError> {
        if length > MAX_BYTES_FOR_BUFFER {
            return Err(DictError::MemoryError);
        }
        if (start_offset + length) > self.ufile_length {
            return Err(DictError::IoError(io::Error::new(io::ErrorKind::UnexpectedEof, "a \
                      seek beyond the end of uncompressed data was requested")));
        }
        let mut data = Vec::new();
        for chunk in self.get_chunks_for(start_offset, length)? {
            let definition = &self.dzdict.as_slice()[chunk.offset..chunk.offset + chunk.length];
            data.push(self.inflate(definition)?);
        };

        // cut definition, convert to string
        let cut_front = start_offset as usize % self.uchunk_length;
        // join the chunks to one vector, only keeping the content of the definition
        let data = match data.len() {
            0 => panic!(),
            1 => data[0][cut_front .. cut_front + length as usize].to_vec(),
            n => {
                let mut tmp = data[0][cut_front..].to_vec();
                // first vec has been inserted into tmp, therefore skip first and last chunk, too
                for text in data.iter().skip(1).take(n-2) {
                    tmp.extend_from_slice(text);
                }
                // add last chunk to tmp, omitting stuff after word definition end
                let remaining_bytes = (length as usize + cut_front) % self.uchunk_length;
                tmp.extend_from_slice(&data[n-1][..remaining_bytes]);
                tmp
            },
        };
        Ok(String::from_utf8(data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::*;

    fn load_resource(name: &str) -> Vec<u8> {
        let mut path = ::std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("assets");
        path.push(name);
        let f = ::std::fs::File::open(path).unwrap();
        let mut data = Vec::new();
        f.read_to_end(&mut data);
        data
    }

    #[test]
    fn test_number_of_parsed_chunks_is_correct() {
        let rsrc = load_resource("lat-deu.dict.dz");
        let d = DictReaderDz::new(rsrc.as_slice()).unwrap();
        assert_eq!(d.chunk_offsets.len(), 7);
    }
}

