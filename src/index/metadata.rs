use super::Location;

/// The special metadata entries that we care about.
///
/// These entries should appear close to the beginning of the index file.
#[derive(Debug, Default)]
pub struct Metadata {
    pub info: Option<String>,
    pub short_name: Option<String>,
    pub url: Option<String>,
    pub all_chars: bool,
    pub case_sensitive: bool,
    pub should_normalize: bool,
}

/// The locations of the special metadata entries.
///
/// # Note
/// Used during initial dictionary parsing.
#[derive(Debug, Default)]
pub struct MetadataIndex {
    /// Read from 00-database-info
    pub info: Option<Location>,

    /// Read from 00-database-short
    pub short_name: Option<Location>,

    /// Read from 00-database-url
    pub url: Option<Location>,

    /// Read from 00-database-allchars
    ///
    /// # Note
    /// Only check for the existence of the metadata entry.
    pub all_chars: bool,

    /// Read from 00-database-case-sensitive
    ///
    /// # Note
    /// Only check for the existence of the metadata entry.
    pub case_sensitive: bool,

    /// Read from 00-database-dictfmt-X.Y.Z
    ///
    /// # Note
    /// Only check for the existence of the metadata entry.
    pub should_normalize: bool,
}

