mod parsing;
mod reader;
mod error;
mod metadata;
use levenshtein::levenshtein;
pub use reader::IndexReader;
pub use error::IndexError;
pub use metadata::Metadata;

use crate::{DictError, DictReader};
use std::{io::{BufRead, Seek, SeekFrom}, ops::Range};
use IndexError::*;
use unidecode::unidecode;

pub struct Index<R: BufRead + Seek> {
    pub reader: R,
    pub entries: Vec<Entry>,
    pub metadata: Metadata,
    pub loaded: bool,
}

/// Location of the headword within the dict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub offset: u64,
    pub size: u64,
}

/// An index entry containing the headword, location and, optionally, the original headword.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub headword: String,
    pub location: Location,
    pub original: Option<String>,
}

impl<R: BufRead + Seek> Index<R> {
    /// Creates a new Index and reads its full metadata.
    pub fn new_full(mut reader: R, dict: &mut Box<dyn DictReader>) -> Result<Self, DictError> {
        let mut metadata = Metadata::default();
        let metadata_index = parsing::parse_metadata(&mut reader)?;

        // Metadata is broken (contains junk chars) if we don't remap it
        let remap = |def: String| {
            let start = def.find('\n').filter(|pos| *pos < def.len() - 1).unwrap_or(0);
            def[start..].trim().to_string()
        };

        // Extract dict info
        if let Some(info) = metadata_index.info {
            metadata.info = Some(dict.fetch_definition(info).map(remap)?);
        }

        // Extract dict short name
        if let Some(short_name) = metadata_index.short_name {
            metadata.short_name = Some(dict.fetch_definition(short_name).map(remap)?);
        }

        // Extract dict url
        if let Some(url) = metadata_index.url {
            metadata.url = Some(dict.fetch_definition(url).map(remap)?);
        }

        // Pass all the other options
        metadata.all_chars = metadata_index.all_chars;
        metadata.case_sensitive = metadata_index.case_sensitive;
        metadata.should_normalize = metadata_index.should_normalize;

        Ok(Self {
            reader,
            entries: Default::default(),
            metadata,
            loaded: false,
        })
    }

    /// Creates a new Index and reads only its basic metadata.
    pub fn new(mut reader: R) -> Result<Self, IndexError> {
        let mut metadata = Metadata::default();
        let metadata_index = parsing::parse_metadata(&mut reader)?;

        // Pass all the other options
        metadata.all_chars = metadata_index.all_chars;
        metadata.case_sensitive = metadata_index.case_sensitive;
        metadata.should_normalize = metadata_index.should_normalize;

        Ok(Self {
            reader,
            entries: Default::default(),
            metadata,
            loaded: false,
        })
    }
}

impl<R: BufRead + Seek> Index<R> {
    fn load_entries(&mut self) -> Result<(), IndexError> {
        // Reset reading to the start
        self.reader.seek(SeekFrom::Start(0))?;

        let mut entries = parsing::parse(&mut self.reader)?;
        if self.metadata.should_normalize {
            normalize(&mut entries, &self.metadata);
        }
        self.entries = entries;

        Ok(())
    }
}

impl<R: BufRead + Seek> IndexReader for Index<R> {
    fn find(&mut self, headword: &str, fuzzy: bool, relaxed: bool) -> Result<Vec<Entry>, IndexError> {
        if !self.loaded {
            self.load_entries()?;
            self.loaded = true;
        }

        // Normalize query according to the metadata
        let mut headword = headword.to_string();
        normalize_headword(&mut headword, &self.metadata);
        let headword: &str = headword.trim();

        if fuzzy {
            let results: Vec<Entry> = self.entries
                .iter()
                .filter(|entry| {
                    if relaxed {
                        let transliterated = unidecode(&entry.headword);
                        levenshtein(headword, transliterated.trim()) <= 1
                    } else {
                        levenshtein(headword, &entry.headword) <= 1
                    }
                })
                .cloned()
                .collect();

            if results.is_empty() { return Err(WordNotFound(headword.into())) }

            Ok(results)
        } else if let Ok(pivot) = self.entries.binary_search_by(|entry| {
                if relaxed {
                    let transliterated = unidecode(&entry.headword);
                    transliterated.trim().cmp(headword)
                } else {
                    entry.headword.as_str().cmp(headword)
                }
            }) {
            let mut results = Vec::new();
            
            // Search for all matching headwords left of the word (alphabetically)
            for i in 0..pivot {
                if relaxed && unidecode(&self.entries[i].headword) != headword { break }
                else if self.entries[i].headword != headword { break }
                results.push(self.entries[i].clone());
            }

            results.push(self.entries[pivot].clone());

            // Search for all matching headwords right of the word (alphabetically)
            for i in pivot + 1..self.entries.len() {
                if relaxed && unidecode(&self.entries[i].headword) != headword { break }
                else if self.entries[i].headword != headword { break }
                results.push(self.entries[i].clone());
            }

            Ok(results)
        } else {
            Err(WordNotFound(headword.into()))
        }
    }

    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

fn normalize(entries: &mut [Entry], metadata: &Metadata) {
    for entry in entries.iter_mut() {
        let old_headword = entry.headword.clone();

        normalize_headword(&mut entry.headword, metadata);

        let original = if old_headword != entry.headword {
            Some(old_headword)
        } else {
            None
        };

        entry.original = original;
    }
}

fn normalize_headword(headword: &mut String, metadata: &Metadata) {
    // Remove all non-alphanumeric and whitespace chars
    if !metadata.all_chars {
        *headword = headword.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect();
    }

    // Convert to lowercase if not case-sensitive
    if !metadata.case_sensitive {
        *headword = headword.to_lowercase();
    }
}

impl Location {
    pub fn new(offset: u64, size: u64) -> Self {
        Self {
            offset,
            size,
        }
    }

    pub fn as_range(&self) -> Range<u64> {
        self.offset..self.offset + self.size
    }
}

