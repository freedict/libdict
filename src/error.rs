use std::io;
use std::string::FromUtf8Error;

/// Error type, representing the errors which can be returned by the libdict library.
///
/// This enum represents a handful of custom errors and wraps `io:::Error` and
/// `string::FromUtf8Error`.
#[derive(Debug, thiserror::Error)]
pub enum DictError {
    /// Invalid character within the index file. Contains detailed positions within the index file.
    #[error("Invalid character '{0}' found on line: {1} at position {2}.")]
    InvalidCharacter(char, usize, usize),

    /// Occurs whenever a line in an index file misses a column.
    #[error("Not enough tab-separated columns in index file, expected 3. Line: {0}")]
    MissingColumnInIndex(usize),

    /// Invalid file format. Contains additional context of the error.
    #[error("Encountered an invalid file format. Context: {0:?}")]
    InvalidFileFormat(String),

    /// This reports a malicious/malformed index file, which requests a buffer which is too large.
    #[error("Requested too much memory. Headword definitions are never larger than 1 MB. The index file is malicious or malformed.")]
    MemoryError,

    /// This reports words which are not present in the dictionary.
    #[error("Word \"{0}\" not found.")]
    WordNotFound(String),

    /// A wrapped io::Error.
    #[error("Encountered an IO error.")]
    IoError(#[from] io::Error),

    /// A wrapped string::FromUtf8Error.
    #[error("Encountered a UTF-8 error.")]
    Utf8Error(#[from] FromUtf8Error),

    /// Errors thrown by the flate2 crate - not really descriptive errors, though.
    #[error("Encountered a decompression error.")]
    Deflate(#[from] flate2::DecompressError),
}
