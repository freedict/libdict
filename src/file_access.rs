//! File Access Wrappers
//!
//! To ease the implementation of the actual `.dict.dz` parser and to decouple it from the
//! underlying file access method, this module provides wrappers for various types of indexeable
//! data structures. One is the memory mapped file, which can be indexed and sliced through a
//! mapper called [MmappedFileSlice](struct.MmappedDict.html). Other wrappers can be trivially
//! implemented for instance for direct file access withour memory mapping or for network
//! transparency. Please not that the current design also allows using a `Vec<u8>` as data source
//! without any wrapper.
use memmap;
use std::ops::{Index, Range};
use std::path::Path;

use errors::DictError;

/// Memory mapped file access.
///
/// This data type manages the mapping of a file into main memory and direct slicing and indexing
/// into the file data.
pub struct MmappedFileSlice {
    mmap: memmap::Mmap,
}

impl MmappedFileSlice {
    /// Return a new instance initialised using the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<MmappedFileSlice, DictError> {
        let mmap = memmap::Mmap::open_path(path, memmap::Protection::Read)?;
        Ok(MmappedFileSlice { mmap: mmap })
    }
}

// Ease the extraction from byte ranges from raw memory
impl Index<Range<usize>> for MmappedFileSlice {
    type Output = [u8];

    /// Retrieve the mapped memory region as a byte slice
    fn index(&self, range: Range<usize>) -> &[u8] {
        // Mmap::as_slice() is unsafe because the caller must guarantee that the
        // referenced memory is never concurrently modified. This is ensured
        // because new() always creates a read-only memory map. Thus, it is okay
        // to wrap this call in a safe method.
        unsafe {
            &self.mmap.as_slice()[range]
        }
    }
}

impl Index<usize> for MmappedFileSlice {
    type Output = u8;

    /// Provide access to a single byte from the mapped memory region.
    fn index(&self, idx: usize) -> &Self::Output {
        // see Index<Range<usize>> for justification
        unsafe {
            &self.mmap.as_slice()[idx]
        }
    }
}


