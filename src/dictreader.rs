use std::io;
use std::io::{BufReader, Seek, SeekFrom, Read};
use std::fs::File;

use errors::DictError;

pub static MAX_BYTES_FOR_BUFFER: u64 = 1048576; // no headword definition is larger than 1M

/// .dict file format: either compressed or uncompressed
pub enum DictFormat {
    GzCompressed,
    Raw
}

/// A dictionary .dict or .dict.gz reader
///
/// This type abstracts from the underlying seek operations required for lookup
/// of headwords and provides easy methods to search for a word given a certain
/// offset. It can parse both compressed and uncompressed .dict files.
/// ToDo: dict.gz
pub struct DictReader<B: Read + Seek> {
    dict_data: B,
    dict_format: DictFormat
} 

impl<B: Read + Seek> DictReader<B> {
    pub fn new(dict_data: B, dict_fmt: DictFormat) -> DictReader<B> {
        DictReader { dict_data: dict_data, dict_format: dict_fmt }
    }

    pub fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError> {
        if length > MAX_BYTES_FOR_BUFFER {
            return Err(DictError::MemoryError);
        }
        match self.dict_format {
            DictFormat::Raw => {
                let _ = try!(self.dict_data.seek(SeekFrom::Start(start_offset)));
                let mut read_data = vec![0; length as usize];
                let bytes_read = try!(self.dict_data.read(read_data.as_mut_slice())) as u64;
                if bytes_read != length { // reading from end of file?
                    return Err(DictError::IoError(io::Error::new(
                                io::ErrorKind::UnexpectedEof, "seek beyond end of file")));
                }
                Ok(try!(String::from_utf8(read_data)))
            },
            _ => panic!("other formats than raw not implemented"),
        }
    }
}

pub fn from_file(path: &str) -> Result<DictReader<BufReader<File>>, DictError> {
    let fmt = match path {
        x if x.ends_with(".dict.dz") => DictFormat::GzCompressed,
        y if y.ends_with(".dict") => DictFormat::Raw,
        _ => return Err(DictError::InvalidFileFormat(
                "unknown file suffix".to_string(), Some(path.to_string())))
    };
    match fmt {
        DictFormat::Raw => {
            let f = try!(File::open(path));
            Ok(DictReader { dict_data: BufReader::new(f), dict_format: fmt })
        },
        DictFormat::GzCompressed => panic!("Gz not implemented yet"),
    }
}

