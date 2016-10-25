use std::io::{BufRead, BufReader};
use std::fs::File;

use errors::DictError;

/// .dict file format: either compressed or uncompressed
enum DictFormat {
    GzCompressed,
    Raw
}

/// A dictionary .dict or .dict.gz reader
///
/// This type abstracts from the underlying seek operations required for lookup
/// of headwords and provides easy methods to search for a word given a certain
/// offset. It can parse both compressed and uncompressed .dict files.
/// ToDo: dict.gz
pub struct DictReader<B: BufRead> {
    dict_data: B,
    dict_format: DictFormat
} 

impl<B: BufRead> DictReader<B> {
    pub fn new(dict_data: B, fmt: DictFormat) -> DictReader<B> {
        DictReader { dict_data: dict_data, dict_format: fmt }
    }

    pub fn from_file(path: &str) -> Result<DictReader<B>, DictError> {
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
}
