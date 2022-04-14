use super::DictError;

pub trait DictReader {
    fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError>;
}

/// Limit size of a word buffer
///
/// Headword definitions are never larger than 1 MB, so prevent malicious or malformed index files
/// from requesting too much memory for a translation.
pub const MAX_BYTES_FOR_BUFFER: u64 = 1_048_576;
