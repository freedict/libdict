use crate::index::Location;

use super::{DictError, DictReader, MAX_BYTES_FOR_BUFFER};
use rassert_rs::rassert;
use std::io::{self, Read, Seek, SeekFrom};
use DictError::*;

/// Uncompressed Dict reader
///
/// This reader can read uncompressed .dict files.
pub struct Uncompressed<R: Read + Seek> {
    pub(crate) reader: R,
    pub(crate) length: u64,
}

impl<R: Read + Seek> Uncompressed<R> {
    pub fn new(mut reader: R) -> Result<Self, DictError> {
        let length = reader.seek(SeekFrom::End(0))?;

        Ok(Self { reader, length })
    }
}

impl<B: Read + Seek> DictReader for Uncompressed<B> {
    fn fetch_definition(&mut self, location: Location) -> Result<String, DictError> {
        rassert!(location.size <= MAX_BYTES_FOR_BUFFER, MemoryError);
        rassert!(location.offset + location.size <= self.length, IoError(io::Error::new(io::ErrorKind::UnexpectedEof,
            "Seek beyond the end of uncompressed data was requested."
        )));

        self.reader.seek(SeekFrom::Start(location.offset))?;
        let mut read_data = vec![0; location.size as usize];
        let bytes_read = self.reader.read(&mut read_data)? as u64;
        rassert!(bytes_read == location.size, IoError(io::Error::new(io::ErrorKind::UnexpectedEof,
            "Seek beyond end of file"
        )));

        Ok(String::from_utf8(read_data)?)
    }
}

