use std::io;
use std::io::{BufReader, Seek, SeekFrom, Read};
use std::fs::File;

use errors::DictError;

pub static MAX_BYTES_FOR_BUFFER: u64 = 1048576; // no headword definition is larger than 1M

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
    /// Get a new DictReader from a BufReader.
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


pub fn load_dict(path: &str) -> Result<Box<DictReader>, DictError> {
    if path.ends_with(".gz") {
        panic!("unimplemented");
    } else {
        let reader = BufReader::new(File::open(path)?);
        Ok(Box::new(DictReaderRaw::new(reader)))
    }
}

