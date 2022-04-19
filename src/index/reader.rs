use super::{IndexError, Entry, Metadata};

/// Generic index reader trait
///
/// # Note
/// Mainly used so that the main `Dict` struct wouldn't be littered
/// with generics (since Index is now a generic over its reader).
pub trait IndexReader {
    fn find(&mut self, headword: &str, fuzzy: bool, relaxed: bool) -> Result<Vec<Entry>, IndexError>;
    fn metadata(&self) -> &Metadata;
}
