use super::{IndexError, Entry, Metadata};

/// Generic index reader trait
///
/// # Note
/// Mainly used so that the main `Dict` struct wouldn't be littered
/// with generics (since Index is now a generic over its reader).
pub trait IndexReader {
    /// Searches the index for the word.
    ///
    /// # Arguments
    ///
    /// * `headword` - The word to search for.
    /// * `fuzzy` - Enables fuzzy searching (up to 1 letter).
    /// * `relaxed` - Enables "relaxed" searching (compares transliterated chars instead of the
    /// original)
    ///
    /// # Expects
    ///
    /// The dictionary index must be in an alphabetical order for the search to work.
    ///
    /// # Returns
    ///
    /// If successful, returns a `Vec` of matching entries, otherwise returns an `IndexError`.
    fn find(&mut self, headword: &str, fuzzy: bool, relaxed: bool) -> Result<Vec<Entry>, IndexError>;

    /// Gets the dictionary's metadata.
    fn metadata(&self) -> &Metadata;
}
