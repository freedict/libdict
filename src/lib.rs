//! A dict format (`*.dict`) reader crate.
//!
//! This crate can read dictionaries in the dict format, as used by dictd. It supports both
//! uncompressed and compressed dictionaries.
//!
//! # Examples
//!
//! The usage is straight forward:
//!
//! ```rust,no_run
//! fn main() {
//!     let index_file = "/usr/share/dictd/freedict-lat-deu.index";
//!     let dict_file = "/usr/share/dictd/freedict-lat-deu.dict.dz";
//!     let mut latdeu = dict::load_dictionary_from_file(dict_file, index_file).unwrap();
//!     // hey: rust!
//!     println!("{}", latdeu.lookup("ferrugo").unwrap());
//! }
//! ```

pub mod dictreader;
pub mod errors;
pub mod indexing;

use self::dictreader::DictReader;
use self::indexing::Index;

use std::path::Path;
use std::collections::HashMap;

/// A dictionary wrapper.
///
/// A dictionary is made up of a `*.dict` or `*.dict.dz` file with the actual content and a
/// `*.index` file with a list of all headwords and with positions in the dict file + length
/// information. It provides a convenience function to look up headwords directly, without caring
/// about the details of the index and the underlying dict format.
/// For an example, please see the [crate documentation](index.html).
pub struct Dictionary {
    dict_reader: Box<dyn DictReader>,
    word_index: HashMap<String, (u64, u64)>
}

impl Dictionary {
    /// Look up a word in a dictionary.
    ///
    /// Words are looked up in the index and then retrieved from the dict file. If no word was
    /// found, `DictError::WordNotFound` is returned. Other errors all result from the parsing of
    /// the underlying files.
    pub fn lookup(&mut self, word: &str) -> Result<String, errors::DictError> {
        let &(start, length) = self.word_index.get(&word.to_lowercase()).ok_or_else(||
                errors::DictError::WordNotFound(word.into()))?;
        self.dict_reader.fetch_definition(start, length)
    }

    /// Check whether a word is contained in the index
    pub fn contains(&self, word: &str) -> bool {
        self.word_index.get(&word.to_lowercase()).is_some()
    }

    /// Case-sensitive member check.
    ///
    /// This will check whether the given word is contained in the index, without checking whether
    /// it's lower case or not. This can help to avoid an additional allocation, if the caller can
    /// be sure that the string is already lower case.
    pub fn contains_unchecked(&self, word: &str) -> bool {
        self.word_index.get(word).is_some()
    }

    /// Get the short name.
    ///
    /// This returns the short name of a dictionary. This corresponds to the
    /// value passed to the `-s` option of `dictfmt`.
    pub fn short_name(&mut self) -> Result<String, errors::DictError> {
        self.lookup("00-database-short")
            .or_else(|_| self.lookup("00databaseshort"))
            // Some dictionaries contain 00-database-short in their entry, others don't:
            .map(|s| match s.find("short") {
                Some(idx) => s[idx + 5..].trim(),
                None => s.trim(),
            }.to_string())
    }
}

/// Load dictionary from given paths
///
/// A dictionary is made of an index and a dictionary (data) file, both are opened from the given
/// input file names. Gzipped files with the suffix `.dz` will be handled automatically.
pub fn load_dictionary_from_file<P: AsRef<Path>>(content_fn: P, index_fn: P) -> Result<Dictionary,
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
pub fn load_dictionary(content: Box<dyn DictReader>, index: Index) -> Dictionary {
    Dictionary { dict_reader: content, word_index: index }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_dictionary() -> Result<Dictionary, errors::DictError> {
        let path = ::std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                                        .join("tests/assets");
        load_dictionary_from_file(path.join("lat-deu.dict.dz"),
                                  path.join("lat-deu.index"))
    }

    #[test]
    fn test_getting_short_name() {
        let mut dict = example_dictionary().unwrap();
        assert_eq!(dict.short_name().ok(),
                   Some("Latin - German FreeDict dictionary ver. 0.4".to_string()));
    }
}
