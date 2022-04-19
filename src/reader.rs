use crate::index::Location;

use super::DictError;

pub trait DictReader {
    /// Reads the bytes from `[offset, offset + size>` and returns them as a string. Alternatively,
    /// returns a `DictError`.
    ///
    /// # Arguments
    ///
    /// * `location` - The location in the reader to read the definition from.
    fn fetch_definition(&mut self, location: Location) -> Result<String, DictError>;
}

/// Limit size of a word buffer
///
/// Headword definitions are never larger than 1 MB, so prevent malicious or malformed index files
/// from requesting too much memory for a translation.
pub const MAX_BYTES_FOR_BUFFER: u64 = 1_048_576;
