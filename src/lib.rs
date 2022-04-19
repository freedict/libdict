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
//!
//!     // Prints out the results of the lookup
//!     println!("{:?}", latdeu.lookup("ferrugo", false, false).unwrap());
//! }
//! ```

pub mod compressed;
mod error;
pub mod index;
mod reader;
mod uncompressed;
pub use compressed::Compressed;
pub use error::DictError;
use index::{IndexReader, Metadata};
pub use reader::{DictReader, MAX_BYTES_FOR_BUFFER};
pub use uncompressed::Uncompressed;
pub use index::{Index, IndexError};

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
    pub(crate) dict: Box<dyn DictReader>,
    pub(crate) index: Box<dyn IndexReader>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LookupResult {
    pub headword: String,
    pub definition: String,
}

impl Dict {
    /// Create a `Dict` from a pair of .dict and .index files.
    pub fn from_file<P: AsRef<Path>>(dict_path: P, index_path: P) -> Result<Self, DictError> {
        let dict_reader = BufReader::new(File::open(&dict_path)?);
        let index_reader = BufReader::new(File::open(&index_path)?);

        let mut dict: Box<dyn DictReader> = if dict_path.as_ref().extension() == Some(OsStr::new("dz")) {
            Box::new(Compressed::new(dict_reader)?)
        } else {
            Box::new(Uncompressed::new(dict_reader)?)
        };

        let index = Box::new(Index::new_full(index_reader, &mut dict)?);

        Ok(Self {
            dict,
            index,
        })
    }

    /// Create a `Dict` from already existing readers.
    pub fn from_existing(dict: Box<dyn DictReader>, index: Box<dyn IndexReader>) -> Result<Self, DictError> {
        Ok(Self { dict, index })
    }

    /// Look up a word in a dictionary.
    ///
    /// # Arguments
    ///
    /// * `word` - Word to search for
    /// * `fuzzy` - Enables fuzzy search (up to one letter)
    /// * `relaxed` - Enables relaxed search mode (no need to match diacritics and other special
    /// letters)
    ///
    /// # Returns
    /// `WordNotFound` if the word wasn't found in the dictionary, parsing errors or, otherwise,
    /// the list of words that match the search query.
    pub fn lookup(&mut self, word: &str, fuzzy: bool, relaxed: bool) -> Result<Vec<LookupResult>, DictError> {
        let entries = self.index.find(word, fuzzy, relaxed)?;

        let mut results = Vec::new();
        for entry in entries {
            results.push(LookupResult {
                headword: entry.original.unwrap_or(entry.headword),
                definition: self.dict.fetch_definition(entry.location)?,
            });
        }

        Ok(results)
    }

    /// Get the metadata of the dictionary.
    pub fn metadata(&self) -> &Metadata {
        self.index.metadata()
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

    fn custom_dictionary(dict_path: &str, index_path: &str) -> Result<Dict, DictError> {
        let dict = get_asset_path().join(dict_path);
        let index = get_asset_path().join(index_path);

        Dict::from_file(dict, index)
    }

    fn lookup_dict_fuzzy(dict: &mut Dict, word: &str, expected: &Vec<LookupResult>) {
        let results = dict.lookup(word, true, false).unwrap();
        assert_eq!(&results, expected);
    }

    fn lookup_dict_exact(dict: &mut Dict, word: &str, expected: &Vec<LookupResult>) {
        let results = dict.lookup(word, false, false).unwrap();
        assert_eq!(&results, expected);
    }
    
    fn lookup_dict_relaxed(dict: &mut Dict, word: &str, expected: &Vec<LookupResult>) {
        let results = dict.lookup(word, false, true).unwrap();
        assert_eq!(&results, expected);
    }



    #[test]
    fn test_getting_short_name() {
        let dict = example_dictionary().unwrap();

        assert_eq!(
            dict.metadata().short_name,
            Some("Latin - German FreeDict dictionary ver. 0.4".to_string())
        );
    }

    #[test]
    fn test_number_of_parsed_chunks_is_correct() {
        let dict_file = load_resource("lat-deu.dict.dz");
        let reader = Compressed::new(dict_file).unwrap();

        assert_eq!(reader.chunk_offsets.len(), 7);
    }

    #[test]
    fn test_dictionary_lookup_case_insensitive() {
        let mut dict = custom_dictionary("case_insensitive_dict.dict", "case_insensitive_dict.index").unwrap();
        let expected = vec![
            LookupResult { headword: "bar".into(), definition: "Bar\ntest for case-sensitivity\n".into() },
        ];

        lookup_dict_exact(&mut dict, "bar", &expected);
        lookup_dict_exact(&mut dict, "Bar", &expected);

        let expected = vec![
            LookupResult { headword: "straße".into(), definition: "straße\ntest for non-latin case-sensitivity\n".into() },
        ];

        lookup_dict_exact(&mut dict, "straße", &expected);
    }

    #[test]
    fn test_dictionary_lookup_case_insensitive_fuzzy() {
        let mut dict = custom_dictionary("case_insensitive_dict.dict", "case_insensitive_dict.index").unwrap();
        let expected = vec![
            LookupResult { headword: "bar".into(), definition: "Bar\ntest for case-sensitivity\n".into() },
        ];

        lookup_dict_fuzzy(&mut dict, "ba", &expected);
    }

    #[test]
    fn test_dictionary_lookup_case_sensitive() {
        let mut dict = custom_dictionary("case_sensitive_dict.dict", "case_sensitive_dict.index").unwrap();
        let expected = vec![
            LookupResult { headword: "Bar".into(), definition: "Bar\ntest for case-sensitivity\n".into() },
        ];

        lookup_dict_exact(&mut dict, "Bar", &expected);

        let expected = vec![
            LookupResult { headword: "straße".into(), definition: "straße\ntest for non-latin case-sensitivity\n".into() },
        ];

        lookup_dict_exact(&mut dict, "straße", &expected);

        assert!(dict.lookup("bar", false, false).is_err());
        assert!(dict.lookup("strasse", false, false).is_err());
    }

    #[test]
    fn test_dictionary_lookup_case_sensitive_fuzzy() {
        let mut dict = custom_dictionary("case_sensitive_dict.dict", "case_sensitive_dict.index").unwrap();
        let expected = vec![
            LookupResult { headword: "Bar".into(), definition: "Bar\ntest for case-sensitivity\n".into() },
        ];

        lookup_dict_fuzzy(&mut dict, "Ba", &expected);

        assert!(dict.lookup("ba", true, false).is_err());
    }

    #[test]
    fn test_dictionary_lookup_relaxed() {
        let mut dict = custom_dictionary("case_insensitive_dict.dict", "case_insensitive_dict.index").unwrap();
        let expected = vec![
            LookupResult { headword: "straße".into(), definition: "straße\ntest for non-latin case-sensitivity\n".into() },
        ];

        lookup_dict_relaxed(&mut dict, "strasse", &expected);
    }
}

