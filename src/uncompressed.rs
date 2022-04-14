use super::{DictError, DictReader, MAX_BYTES_FOR_BUFFER};
use rassert_rs::rassert;
use std::io::{self, Read, Seek, SeekFrom};
use DictError::*;

/// Uncompressed Dict reader
///
/// This reader can read uncompressed .dict files.
pub struct Uncompressed<B: Read + Seek> {
    pub(crate) buf: B,
    pub(crate) length: u64,
}

impl<B: Read + Seek> Uncompressed<B> {
    pub fn new(mut buf: B) -> Result<Self, DictError> {
        let length = buf.seek(SeekFrom::End(0))?;

        Ok(Self { buf, length })
    }
}

impl<B: Read + Seek> DictReader for Uncompressed<B> {
    fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError> {
        rassert!(length <= MAX_BYTES_FOR_BUFFER, MemoryError);
        rassert!(start_offset + length <= self.length, IoError(io::Error::new(io::ErrorKind::UnexpectedEof,
            "Seek beyond the end of uncompressed data was requested."
        )));

        self.buf.seek(SeekFrom::Start(start_offset))?;
        let mut read_data = vec![0; length as usize];
        let bytes_read = self.buf.read(&mut read_data)? as u64;
        rassert!(bytes_read == length, IoError(io::Error::new(io::ErrorKind::UnexpectedEof,
            "Seek beyond end of file"
        )));

        Ok(String::from_utf8(read_data)?)
    }
}

