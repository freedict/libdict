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
//! use dict::Dict;
//!
//! fn main() {
//!     let index_file = "/usr/share/dictd/freedict-lat-deu.index";
//!     let dict_file = "/usr/share/dictd/freedict-lat-deu.dict.dz";
//!     let mut latdeu = Dict::from_file(dict_file, index_file).unwrap();
//!     // hey: rust!
//!     println!("{}", latdeu.lookup("ferrugo").unwrap());
//! }
//! ```

pub mod compressed;
mod error;
pub mod index;
mod reader;
mod uncompressed;
pub use compressed::Compressed;
pub use error::DictError;
pub use reader::{DictReader, MAX_BYTES_FOR_BUFFER};
pub use uncompressed::Uncompressed;

use crate::index::Index;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// A dictionary wrapper.
///
/// A dictionary is made up of a `*.dict` or `*.dict.dz` file with the actual content and a
/// `*.index` file with a list of all headwords and with positions in the dict file + length
/// information. It provides a convenience function to look up headwords directly, without caring
/// about the details of the index and the underlying dict format.
/// For an example, please see the [crate documentation](index.html).
pub struct Dict {
    pub(crate) reader: Box<dyn DictReader>,
    pub(crate) index: Index,
}

impl Dict {
    pub fn from_file<P: AsRef<Path>>(dict_path: P, index_path: P) -> Result<Self, DictError> {
        let dict_reader = BufReader::new(File::open(&dict_path)?);
        let index_reader = BufReader::new(File::open(&index_path)?);

        let reader: Box<dyn DictReader> =
            if dict_path.as_ref().extension() == Some(OsStr::new("dz")) {
                Box::new(Compressed::new(dict_reader)?)
            } else {
                Box::new(Uncompressed::new(dict_reader)?)
            };

        Ok(Self {
            reader,
            index: Index::new(index_reader)?,
        })
    }

    pub fn from_existing(reader: Box<dyn DictReader>, index: Index) -> Result<Self, DictError> {
        Ok(Self { reader, index })
    }

    /// Look up a word in a dictionary.
    ///
    /// Words are looked up in the index and then retrieved from the dict file. If no word was
    /// found, `DictError::WordNotFound` is returned. Other errors all result from the parsing of
    /// the underlying files.
    pub fn lookup(&mut self, word: &str) -> Result<String, DictError> {
        let &(start, length) = self
            .index
            .words
            .get(&word.to_lowercase())
            .ok_or_else(|| DictError::WordNotFound(word.into()))?;
        self.reader.fetch_definition(start, length)
    }

    /// Check whether a word is contained in the index
    pub fn contains(&self, word: &str) -> bool {
        self.index.words.get(&word.to_lowercase()).is_some()
    }

    /// Case-sensitive member check.
    ///
    /// This will check whether the given word is contained in the index, without checking whether
    /// it's lower case or not. This can help to avoid an additional allocation, if the caller can
    /// be sure that the string is already lower case.
    pub fn contains_unchecked(&self, word: &str) -> bool {
        self.index.words.get(word).is_some()
    }

    /// Get the short name.
    ///
    /// This returns the short name of a dictionary. This corresponds to the
    /// value passed to the `-s` option of `dictfmt`.
    pub fn short_name(&mut self) -> Result<String, DictError> {
        self.lookup("00-database-short")
            .or_else(|_| self.lookup("00databaseshort"))
            .map(|def| {
                let start = if def.starts_with("00-database-short") {
                    17
                } else {
                    0
                };
                def[start..].trim().to_string()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::PathBuf;

    fn get_asset_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("assets")
    }

    fn get_resource(name: &str) -> PathBuf {
        get_asset_path().join(name)
    }

    fn load_resource(name: &str) -> File {
        let res = get_resource(name);
        File::open(res).unwrap()
    }

    fn example_dictionary() -> Result<Dict, DictError> {
        let dict = get_asset_path().join("lat-deu.dict.dz");
        let index = get_asset_path().join("lat-deu.index");

        Dict::from_file(dict, index)
    }

    #[test]
    fn test_getting_short_name() {
        let mut dict = example_dictionary().unwrap();

        assert_eq!(
            dict.short_name().ok(),
            Some("Latin - German FreeDict dictionary ver. 0.4".to_string())
        );
    }

    #[test]
    fn test_number_of_parsed_chunks_is_correct() {
        let dict_file = load_resource("lat-deu.dict.dz");
        let reader = Compressed::new(dict_file).unwrap();

        assert_eq!(reader.chunk_offsets.len(), 7);
    }
}

