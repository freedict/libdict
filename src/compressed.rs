use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use rassert_rs::rassert;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};

use super::{DictError, DictReader, MAX_BYTES_FOR_BUFFER};
use DictError::*;

/// Compressed (gzip) Dict reader
///
/// This reader can read compressed .dict files with the file name suffix .dz.
/// This format is documented in RFC 1952 and in `man dictzip`. An example implementation can be
/// found in the dict daemon (dictd) in `data.c`.
pub struct Compressed<B: Read + Seek> {
    /// Compressed buffer
    pub(crate) buf: B,

    /// Length of an uncompressed chunk
    pub(crate) uchunk_length: usize,

    /// End of compressed data
    pub(crate) end_compressed_data: u64,

    /// Offsets in file where new compressed chunks start
    pub(crate) chunk_offsets: Vec<u64>,

    /// Total size of uncompressed file
    pub(crate) ufile_length: u64,
}

/// Byte mask to query for existence of FEXTRA field in the flags byte of a `.dz` file
pub const GZ_FEXTRA: u8 = 0b0000_0100;

/// Byte mask to query for the existence of a file name in a `.dz` file
pub const GZ_FNAME: u8 = 0b0000_1000;

/// Byte mask to query for the existence of a comment in a `.dz` file
pub const GZ_COMMENT: u8 = 0b0001_0000;

/// Byte mask to detect that a comment is contained in a `.dz` file
pub const GZ_FHCRC: u8 = 0b0000_0010;

/// A (gz) chunk, representing length and offset within the compressed file
#[derive(Debug)]
struct Chunk {
    offset: u64,
    length: usize,
}

impl<B: Read + Seek> Compressed<B> {
    pub fn new(mut buf: B) -> Result<Self, DictError> {
        let mut header = vec![0; 12];

        // Check header
        buf.read_exact(&mut header)?;
        rassert!(&header[0..2] == &[0x1F, 0x8B], InvalidFileFormat("Not in gzip format".into()));

        // Check for FEXTRA flag
        let flags = header[3];
        rassert!(flags & GZ_FEXTRA != 0, InvalidFileFormat("Extra flag (FLG.FEXTRA) not set. Not in gzip + dzip format.".into()));

        // Read length of FEXTRA field
        let xlen = LittleEndian::read_u16(&header[10..12]);

        // Read FEXTRA field
        let mut fextra = vec![0; xlen as usize];
        buf.read_exact(&mut fextra)?;
        rassert!(&fextra[0..2] == b"RA", InvalidFileFormat("No dictzip info found in FEXTRA header (behind XLEN, in SI1SI2 fields".into()));

        // Check subfield length
        let subfield_length = LittleEndian::read_u16(&fextra[2..4]);
        rassert!(subfield_length == xlen - 4, InvalidFileFormat(
            "The length of the subfield should be the same as the FEXTRA field, \
            ignoring the additional length information and the file format identification".into()
        ));

        // Check dictzip version
        let version = LittleEndian::read_u16(&fextra[4..6]);
        rassert!(version == 1, InvalidFileFormat("Unimplemented dictzip version, only version 1 supported".into()));

        // Before compression, the file is split into evenly-sized chunks and the
        // size information is put right after the version information
        let uchunk_length = LittleEndian::read_u16(&fextra[6..8]) as usize;
        let chunk_count = LittleEndian::read_u16(&fextra[8..10]);
        rassert!(chunk_count != 0, InvalidFileFormat("No compressed chunks in file or broken header information".into()));

        // Compute number of possible chunks which would fit into the FEXTRA field.
        // Used for validity check, first 10 bytes of FEXTRA are header information,
        // the rest are 2-byte, little-endian numbers.
        let max_chunks = ((fextra.len() - 10) / 2) as u16;
        rassert!(max_chunks == chunk_count, InvalidFileFormat(format!(
            "Expected {} chunks according to dictzip header, but the FEXTRA field accomodate {}. Possibly broken file.", 
            chunk_count, max_chunks
        )));

        // If filename bit set, skip nul-terminated filename
        if flags & GZ_FNAME != 0 {
            while buf.read_u8()? != b'\0' {}
        }

        // Skip comment
        if flags & GZ_COMMENT != 0 {
            while buf.read_u8()? != b'\0' {}
        }

        // Skip CRC bytes
        if flags & GZ_FHCRC != 0 {
            buf.seek(SeekFrom::Current(2))?;
        }

        // Save length of each compressed chunk
        let mut chunk_offsets = Vec::with_capacity(chunk_count as usize);

        // Save position of last compressed byte
        // Note: This might not be EOF, could be followed by CRC checksum.
        let mut end_compressed_data = buf.seek(SeekFrom::Current(0))?;

        // After the various header bytes parsed above, the list of chunk lengths
        // can be found (slice for easier indexing)
        let chunks_from_header = &fextra[10..(10 + chunk_count * 2) as usize];
        let chunk_sizes = chunks_from_header
            .chunks(2)
            .map(|slice| LittleEndian::read_u16(slice) as u64);

        // Push all chunk offsets
        for size in chunk_sizes {
            chunk_offsets.push(end_compressed_data);
            end_compressed_data += size;
        }

        rassert!(chunk_offsets.len() == chunk_count as usize, InvalidFileFormat(
            "The read number of compressed chunks in the .dz file must be equivalent \
            to the number of chunks actually found in the file".into()
        ));

        // Read uncompressed file length
        buf.seek(SeekFrom::Start(end_compressed_data as u64))?;
        let ufile_length = buf.read_i32::<LittleEndian>()? as u64;

        Ok(Self {
            buf,
            chunk_offsets,
            end_compressed_data,
            uchunk_length,
            ufile_length,
        })
    }

    /// Inflate a dictdz chunk
    fn inflate(&self, data: Vec<u8>) -> Result<Vec<u8>, DictError> {
        let mut decoder = flate2::Decompress::new(false);
        let mut decoded = vec![0; self.uchunk_length];
        decoder.decompress(&data, &mut decoded, flate2::FlushDecompress::None)?;

        Ok(decoded)
    }

    fn get_chunks_for(&self, start_offset: u64, length: u64) -> Result<Vec<Chunk>, DictError> {
        let mut chunks = Vec::new();
        let start = start_offset as usize / self.uchunk_length;
        let end = (start_offset + length) as usize / self.uchunk_length;
        for id in start..=end {
            let offset = self.chunk_offsets[id];
            let length = match self.chunk_offsets.get(id + 1) {
                Some(next) => next - offset,
                None => self.end_compressed_data - offset,
            } as usize;

            chunks.push(Chunk { offset, length });
        }

        Ok(chunks)
    }
}

impl<B: Read + Seek> DictReader for Compressed<B> {
    fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError> {
        rassert!(length <= MAX_BYTES_FOR_BUFFER, MemoryError);
        rassert!(start_offset + length < self.ufile_length, IoError(io::Error::new(io::ErrorKind::UnexpectedEof,
            "Seek beyond the end of uncompressed data was requested."
        )));

        let mut data = Vec::new();
        for chunk in self.get_chunks_for(start_offset, length)? {
            let pos = self.buf.seek(SeekFrom::Start(chunk.offset))?;
            rassert!(pos == chunk.offset, IoError(io::Error::new(io::ErrorKind::Other, format!(
                "Attempted to seek to {} but new position is {}",
                chunk.offset, pos
            ))));

            let mut definition = vec![0; chunk.length];
            self.buf.read_exact(&mut definition)?;
            data.push(self.inflate(definition)?);
        }

        // Cut definition, convert to string
        let cut_front = start_offset as usize % self.uchunk_length;

        let data = match data.len() {
            0 => unreachable!(),
            1 => data[0][cut_front..cut_front + length as usize].to_vec(),
            n => {
                let mut tmp = data[0][cut_front..].to_vec();

                // First vec has been inserted into tmp, therefore skip first and last chunk, too
                for text in data.iter().skip(1).take(n - 2) {
                    tmp.extend_from_slice(text);
                }

                // Add last chunk to tmp, omitting stuff after word definition end
                let remaining_bytes = (length as usize + cut_front) % self.uchunk_length;
                tmp.extend_from_slice(&data[n - 1][..remaining_bytes]);
                tmp
            }
        };

        Ok(String::from_utf8(data)?)
    }
}

