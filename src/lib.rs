pub mod indexing;
pub mod dictreader;
mod errors;

// make errors appear on top level
pub use errors::*;


use std::collections::HashMap;
use std::io::BufReader;
use std::fs::File;


use self::dictreader::{DictReader, DictReaderRaw};

macro_rules! get(
    ($e:expr) => (match $e {
        Some(e) => e,
        None => return None
    })
);


pub struct Dictionary {
    dict_reader: Box<DictReader>,
    word_index: HashMap<String, (u64, u64)>
}

impl Dictionary {
    fn lookup(&mut self, word: &str) -> Option<String> {
        let &(start, length) = get!(self.word_index.get(word));
        self.dict_reader.fetch_definition(start, length).ok()
    }
}

/// Load dictionary from given input
///
/// A dictionary is made of an index and a dictionary (data) file, both are opened from the given
/// input file names. Gzipped files will be handled automatically. ToDo: nimplemented
pub fn load_dictionary(content_fn: &str, index_fn: &str) -> Result<Dictionary,
            errors::DictError> {
    let br = BufReader::new(File::open(content_fn)?);
    let dreader = Box::new(DictReaderRaw::new(br));
    let index = indexing::parse_index_from_file(index_fn)?;
    Ok(Dictionary { dict_reader: dreader, word_index: index })
}

