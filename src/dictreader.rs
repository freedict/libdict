use byteorder::*;
use flate2;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufRead, Cursor, Read, Seek, SeekFrom};

use errors::DictError;

/// limit size of a word buffer, so that  malicious index files cannot request to much memory for a
/// translation
pub static MAX_BYTES_FOR_BUFFER: u64 = 1048576; // no headword definition is larger than 1M

/// Flags for the GZ header to query for certain peroperties:
static GZ_FEXTRA: u8 = 0b00000100; // fextra bit, additional field with information about gzip chunk size
static GZ_FNAME: u8   = 0b00001000; // indicates whether a file name is contained in the archive
static GZ_COMMENT: u8 = 0b00010000; // ndicates, whether a comment is present
static GZ_FHCRC: u8   = 0b00000010; // indicate whether a CRC checksum is present


/// .dict file format: either compressed or uncompressed
/// A dictionary .dict or .dict.gz reader
///
/// This type abstracts from the underlying seek operations required for lookup
/// of headwords and provides easy methods to search for a word given a certain
/// offset. It can parse both compressed and uncompressed .dict files.
/// ToDo: dict.gz
pub trait DictReader {
    fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError>;
}

/// Raw Dict reader
///
/// This reader can read uncompressed .dict files.
pub struct DictReaderRaw<B: Read + Seek> {
    dict_data: B,
}

impl<B: Read + Seek> DictReaderRaw<B> {
    /// Get a new DictReader from a Reader.
    pub fn new(dict_data: B) -> DictReaderRaw<B> {
        DictReaderRaw { dict_data: dict_data }
    }
}

impl<B: Read + Seek> DictReader for DictReaderRaw<B> {
    fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError> {
        if length > MAX_BYTES_FOR_BUFFER {
            return Err(DictError::MemoryError);
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

// ToDo: doc
pub fn load_dict(path: &str) -> Result<Box<DictReader>, DictError> {
    if path.ends_with(".dz") {
        let reader = File::open(path)?;
        Ok(Box::new(DictReaderDz::new(reader)?))
    } else {
        let reader = BufReader::new(File::open(path)?);
        Ok(Box::new(DictReaderRaw::new(reader)))
    }
}


////////////////////////////////////////////////////////////////////////////////
// gzip handling

/// Gzip Dict reader
///
/// This reader can read compressed .dict files. with the file name suffix .dz.
/// This format is documented in RFC 1952 and in `man dictzip`. An example implementation can be
/// found in the dict daemon (dictd) in `data.c`.
pub struct DictReaderDz<B: Read + Seek> {
    /// compressed DZ dictionary
    dzdict: B,
    /// length of an uncompressed chunk
    uchunk_length: usize,
    /// end of compressed data in file
    end_of_header: usize,
    /// offsets in file where a new compressed chunk starts
    chunk_offsets: Vec<usize>
}

#[derive(Debug)]
struct Chunk {
    offset: usize,
    length: usize,
}

impl<B: Read + Seek> DictReaderDz<B> {
    /// Get a new DictReader from a Reader.
    pub fn new(dzdict: B) -> Result<DictReaderDz<B>, DictError> {
        let mut buffered_dzdict = BufReader::new(dzdict);
        let mut header = vec![0u8; 12];
        buffered_dzdict.read_exact(&mut header)?;
        if header[0..2] != [0x1F, 0x8B] {
            return Err(DictError::InvalidFileFormat("Not in gzip format".into(), None));
        }
    
        let flags = &header[3]; // bitmap of gzip attributes
        if (flags & GZ_FEXTRA) == 0 { // check whether FLG.FEXTRA is set
            return Err(DictError::InvalidFileFormat("Extra flag (FLG.FEXTRA) \
                       not set, not in gzip + dzip format".into(), None));
        }
    
        // read XLEN, length of extra FEXTRA field
        let xlen = LittleEndian::read_u16(&header[10..12]);
    
        // read FEXTRA data
        let mut fextra = vec![0u8; xlen as usize];
        buffered_dzdict.read(&mut fextra)?;

        if fextra[0..2] != ['R' as u8, 'A' as u8] {
            return Err(DictError::InvalidFileFormat("No dictzip info found in FEXTRA \
                    header (behind XLEN, in SI1SI2 fields)".into(), None));
        }
    
        let length_subfield = LittleEndian::read_u16(&fextra[2..4]);
        assert_eq!(length_subfield, xlen - 4, "the length of the subfield \
                   should be the same as the fextra field, ignoring the \
                   additional length information and the file format identification");
        let subf_version = LittleEndian::read_u16(&fextra[4..6]);
        if subf_version != 1 {
             return Err(DictError::InvalidFileFormat("Unimplemented dictzip \
                     version, only ver 1 supported".into(), None));
        }
    
        // before compression, the file is split into evenly-sized chunks and the size information
        // is put right after the version information:
        let uchunk_length = LittleEndian::read_u16(&fextra[6..8]);
        // number of chunks in the file
        let chunk_count = LittleEndian::read_u16(&fextra[8..10]);
        if chunk_count == 0 {
            return Err(DictError::InvalidFileFormat("No compressed chunks in \
                    file or broken header information".into(), None));
        }
    
        // compute number of possible chunks which would fit into the FEXTRA field; used for
        // validity check. first 10 bytes of FEXTRA are header information, the rest are 2-byte,
        // little-endian numbers.
        let numbers_chunks_which_would_fit = ((fextra.len() - 10) / 2) as u16; // each chunk represented by u16 == 2 bytes
        // check that number of claimed chunks fits within given size for subfield
        if numbers_chunks_which_would_fit != chunk_count {
            return Err(DictError::InvalidFileFormat(format!("Expected {} chunks \
                      according to dictzip header, but the FEXTRA field can \
                      accomodate {}; possibly broken file", chunk_count,
                      numbers_chunks_which_would_fit), None));
        }
    
        // if file name bit set, seek beyond the 0-terminated file name, we don't care
        if (flags & GZ_FNAME) != 0 {
            let mut tmp = Vec::new();
            buffered_dzdict.read_until('\0' as u8, &mut tmp)?;
        }
    
        // seek past comment, if any
        if (flags & GZ_COMMENT) != 0 {
            let mut tmp = Vec::new();
            buffered_dzdict.read_until('\0' as u8, &mut tmp)?;
        }
    
        // skip CRC stuff, 2 bytes
        if (flags & GZ_FHCRC) != 0 {
            buffered_dzdict.seek(SeekFrom::Current(2))?;
        }
    
        // save length of each compressed chunk
        let mut chunk_offsets = Vec::with_capacity(chunk_count as usize);
        // save position of last compressed byte (this is NOT EOF, could be followed by CRC checksum)
        let mut end_of_header = buffered_dzdict.seek(SeekFrom::Current(0))? as usize;
        // after the various header bytes parsed above, the list of chunk lengths can be found (slice for easier indexing)
        let chunks_from_header = &fextra[10usize..(10 + chunk_count * 2) as usize];

        // iterate over each 2nd byte, parse u16
        for index in (0..chunks_from_header.len()).filter(|i| (i%2)==0) {
            let index = index as usize;
            let compressed_len = LittleEndian::read_u16(&chunks_from_header[index..(index + 2)]) as usize;
            chunk_offsets.push(end_of_header);
            end_of_header += compressed_len;
        }
   
        assert_eq!(chunk_offsets.len() as u16, chunk_count, "The read number of compressed chunks in \
                the .dz file must be equivalent to the number of chunks actually found in the file.\n");
    
        Ok(DictReaderDz { dzdict: buffered_dzdict.into_inner(),
                chunk_offsets: chunk_offsets,
                end_of_header: end_of_header,
                uchunk_length: uchunk_length as usize })
    }

    // ToDo: return meaningful error if length points to position past EOF
    fn get_chunks_for(&self, start_offset: u64, length: u64) -> Result<Vec<Chunk>, DictError> {
        let mut chunks = Vec::new();
        let start_chunk = start_offset as usize / self.uchunk_length;
        let end_chunk = (start_offset + length) as usize / self.uchunk_length;
        for id in start_chunk..(end_chunk + 1) {
            let chunk_length = match self.chunk_offsets.get(id+1) {
                Some(next) => next - self.chunk_offsets[id],
                None => self.end_of_header - self.chunk_offsets[id],
            };
            chunks.push(Chunk { offset: self.chunk_offsets[id], length: chunk_length });
        }

        Ok(chunks)
    }

    // inflate a dictdz chunk
    fn inflate(&self, data: Vec<u8>) -> Result<Vec<u8>, DictError> {
        let mut decoder = flate2::Decompress::new(false);
        let mut decoded = vec![0u8; self.uchunk_length];
        decoder.decompress(data.as_slice(), decoded.as_mut_slice(), flate2::Flush::None).unwrap();
        Ok(decoded)
    }
}

impl<B: Read + Seek> DictReader for DictReaderDz<B> {
    fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError> {
        if length > MAX_BYTES_FOR_BUFFER {
            return Err(DictError::MemoryError);
        }
        let mut data = Vec::new();
        for chunk in self.get_chunks_for(start_offset, length)? {
            let pos = self.dzdict.seek(SeekFrom::Start(chunk.offset as u64))?;
            if pos != (chunk.offset as u64) {
                return Err(DictError::IoError(io::Error::new(io::ErrorKind::Other, format!(
                        "attempted to seek to {} but new position is {}",
                        chunk.offset, pos))));
            }
            let mut definition = vec![0u8; chunk.length];
            self.dzdict.read_exact(&mut definition)?;
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

    fn load_resource(name: &str) -> ::std::fs::File {
        let mut path = ::std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("assets");
        path.push(name);
        ::std::fs::File::open(path).unwrap()
    }

    #[test]
    fn test_number_of_parsed_chunks_is_correct() {
        let rsrc = load_resource("lat-deu.dict.dz");
        let d = DictReaderDz::new(rsrc).unwrap();
        assert_eq!(d.chunk_offsets.len(), 7);
    }
}

