pub mod dictreader;
pub mod errors;
pub mod indexing;

use self::dictreader::DictReader;

use std::collections::HashMap;

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
    pub fn lookup(&mut self, word: &str) -> Option<String> {
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
    let dreader = dictreader::load_dict(content_fn)?;
    let index = indexing::parse_index_from_file(index_fn)?;
    Ok(Dictionary { dict_reader: dreader, word_index: index })
}

