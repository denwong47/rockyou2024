//! Blocking implementations of the reader.
//!
use crate::config;

/// Memory-mapped file reader, reading the file in chunks.
///
/// This is a synchronous reader, and is used as a baseline for the performance of the
/// asynchronous reader. This is designed to be an [`Iterator`] over the chunks of [`&[u8]`].
pub struct MmapReader {
    mmap: memmap::Mmap,
    pub chunk_size: usize,
}

impl MmapReader {
    /// Create a new instance of the MmapReader using the provided memory-mapped file.
    pub fn new(mmap: memmap::Mmap) -> Self {
        Self {
            mmap,
            chunk_size: config::CHUNK_SIZE,
        }
    }

    /// Set the chunk size for the MmapReader.
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    /// Set the chunk size to split the file evenly into the given number of chunks.
    pub fn with_chunks(mut self, chunks: usize) -> Self {
        self.chunk_size =
            self.mmap.len() / chunks + if self.mmap.len() % chunks > 0 { 1 } else { 0 };
        self
    }

    /// Read the provided [`std::fs::File`] using [`MmapReader`].
    pub fn from_file(file: std::fs::File) -> Self {
        let mmap = unsafe {
            memmap::MmapOptions::new()
                .map(&file)
                .unwrap_or_else(|_| panic!("Could not memory-map the file at {:?}.", file))
        };

        Self::new(mmap)
    }

    /// Read the file at the given path using [`MmapReader`].
    pub fn from_path(path: &str) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(path)?;
        Ok(Self::from_file(file))
    }

    /// Seek from a specific position in the file, until a certain byte is found.
    /// Returns the position after the byte if found, or None otherwise.
    ///
    /// Start of file is deemed to match any bytes; so if `position` is 0,
    /// the [`Self::seek_from`] will always return [`Some`](`0`).
    pub fn seek_from(&self, position: usize, byte: u8) -> Option<usize> {
        if position == 0 {
            return Some(0);
        } else if position >= self.mmap.len() {
            return None;
        }

        self.mmap[position..]
            .iter()
            .enumerate()
            .find_map(|(offset, &b)| (b == byte).then(|| position + offset + 1))
    }

    /// Read the next chunk of bytes from the [`MmapReader`], up to the next matching
    /// byte after the end of the chunk, or the end of the file.
    ///
    /// This does NOT check if the starting position makes any sense.
    pub fn read_from(&self, position: usize, byte: u8) -> Option<&[u8]> {
        if position >= self.mmap.len() {
            return None;
        }

        // End is either the next matching byte, or the end of the file.
        let end = self
            .seek_from(position + self.chunk_size, byte)
            .unwrap_or_else(|| self.mmap.len());
        let chunk = &self.mmap[position..end];

        Some(chunk)
    }

    /// Iterate over the chunks of bytes in the memory-mapped file.
    pub fn iter<const SEP: u8>(&self) -> IterMmapReader<'_, SEP> {
        IterMmapReader {
            reader: self,
            cursor: 0,
        }
    }

    /// Get the length of the memory-mapped file.
    pub fn len(&self) -> usize {
        self.mmap.len()
    }

    /// Get the number of chunks in the memory-mapped file.
    pub fn chunks_count(&self) -> usize {
        self.len() / self.chunk_size
            + if self.len() % self.chunk_size > 0 {
                1
            } else {
                0
            }
    }
}

/// An iterator over the chunks of bytes in a memory-mapped file.
pub struct IterMmapReader<'m, const SEP: u8> {
    reader: &'m MmapReader,
    cursor: usize,
}

impl<'m, const SEP: u8> Iterator for IterMmapReader<'m, SEP> {
    type Item = &'m [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.reader.read_from(self.cursor, SEP);

        if let Some(chunk) = chunk {
            self.cursor += chunk.len();
            Some(chunk)
        } else {
            None
        }
    }
}
