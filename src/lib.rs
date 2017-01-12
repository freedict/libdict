//! A dict format (`*.dict`) reader crate.
//!
//! This crate can read dictionaries in the dict format, as used by dictd. It supports both
//! uncompressed and compressed dictionaries.
//!
//! # Examples
//!
//! The usage is straight forward:
//!
//! ```
//! fn main() {
//!     let index_file = "/usr/share/dictd/freedict-lat-deu.index";
//!     let dict_file = "/usr/share/dictd/freedict-lat-deu.dict.dz";
//!     let mut latdeu = dict::load_dictionary_from_file(dict_file, index_file).unwrap();
//!     // hey: rust!
//!     println!("{}", latdeu.lookup("ferrugo").unwrap());
//! }
//! ```
extern crate byteorder;
extern crate flate2;

pub mod dictreader;
pub mod errors;
pub mod indexing;

use self::dictreader::DictReader;
use self::indexing::Index;

use std::collections::HashMap;

macro_rules! get(
    ($e:expr) => (match $e {
        Some(e) => e,
        None => return None
    })
);

/// A dictionary wrapper.
///
/// A dictionary is made up of a `*.dict` or `*.dict.dz` file with the actual content and a
/// `*.index` file with a list of all headwords and with positions in the dict file + length
/// information. It provides a convenience function to look up headwords directly, without caring
/// about the details of the index and the underlying dict format.
/// For an example, please see the [crate documentation](index.html).
pub struct Dictionary {
    dict_reader: Box<DictReader>,
    word_index: HashMap<String, (u64, u64)>
}

impl Dictionary {
    /// Look up a word in a dictionary.
    ///
    /// Words are looked up in the index and then retrieved from the dict file. If no word was
    /// found, `DictError::WordNotFound` is returned. Other errors all result from the parsing of
    /// the underlying files.
    pub fn lookup(&mut self, word: &str) -> Result<String, errors::DictError> {
        let &(start, length) = self.word_index.get(word).ok_or(errors::DictError::WordNotFound(word.into()))?;
        self.dict_reader.fetch_definition(start, length)
    }
}

/// Load dictionary from given paths
///
/// A dictionary is made of an index and a dictionary (data) file, both are opened from the given
/// input file names. Gzipped files will be handled automatically. ToDo: nimplemented
pub fn load_dictionary_from_file(content_fn: &str, index_fn: &str) -> Result<Dictionary,
            errors::DictError> {
    let dreader = dictreader::load_dict(content_fn)?;
    let index = indexing::parse_index_from_file(index_fn)?;
    Ok(Dictionary { dict_reader: dreader, word_index: index })
}

/// Load dictionary from given [DictReader](dictreader/index.html) and [Index](indexing/type.Index.html).
///
/// A dictionary is made of an index and a dictionary (data). Both are required for look up. This
/// function allows abstraction from the underlying source by only requiring a
/// [dictReader](dictreader) as trait object. This way, dictionaries from RAM or similar can be
/// implemented.
pub fn load_dictionary(content: Box<DictReader>, index: Index) -> Dictionary {
    Dictionary { dict_reader: content, word_index: index }
}

