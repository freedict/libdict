use std::io;

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("No metadata found in index file.")]
    NoMetadataFound,

    /// Reports words which are not present in the dictionary.
    #[error("Word '{0}' not found in the dictionary.")]
    WordNotFound(String),

    /// Invalid character within the index file. Contains detailed positions within the index file.
    #[error("Invalid character '{0}' found on line: {1} at position {2}.")]
    InvalidCharacter(char, usize, usize),

    /// Occurs whenever a line in an index file misses a column.
    #[error("Not enough tab-separated columns in index file, expected at least 3. Line: {0}")]
    MissingColumnInIndex(usize),

    /// A wrapped io::Error.
    #[error("Encountered an IO error.")]
    IoError(#[from] io::Error),
}

