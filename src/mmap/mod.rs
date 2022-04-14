use std::{io, fs::File, path::Path};
use advisory_lock::{AdvisoryFileLock, FileLockMode, FileLockError};
use memmap2::Mmap;

pub struct MmapCursor {
    file: File,
    mmap: Mmap,
    pos: u64,
}

impl MmapCursor {
    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        Ok(Self {
            file,
            mmap,
            pos: 0,
        })
    }

    pub fn from_file(file: File) -> io::Result<Self> {
        let mmap = unsafe { Mmap::map(&file)? };

        Ok(Self {
            file,
            mmap,
            pos: 0,
        })
    }

    pub fn remaining_slice(&self) -> &[u8] {
        let len = self.pos.min(self.mmap.len() as u64);
        &self.mmap[(len as usize)..]
    }
} 

impl io::Read for MmapCursor {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.lock(FileLockMode::Shared).or_else(|e| match e {
            FileLockError::AlreadyLocked => unreachable!(), // File is in a blocking lock, shouldn't happen?
            FileLockError::Io(e) => Err(e),
        })?;

        let n = io::Read::read(&mut self.remaining_slice(), buf)?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl io::Seek for MmapCursor {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.file.lock(FileLockMode::Shared).or_else(|e| match e {
            FileLockError::AlreadyLocked => unreachable!(), // File is in a blocking lock, shouldn't happen?
            FileLockError::Io(e) => Err(e),
        })?;

        let (base_pos, offset) = match pos {
            io::SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            }
            io::SeekFrom::End(n) => (self.mmap.len() as u64, n),
            io::SeekFrom::Current(n) => (self.pos, n),
        };

        let new_pos = if offset >= 0 {
            u64::checked_add(base_pos, offset as u64)
        } else {
            u64::checked_add(base_pos, offset.unsigned_abs())
        };

        match new_pos {
            Some(n) => {
                self.pos = n;
                Ok(n)
            }
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )),
        }
    }
}

