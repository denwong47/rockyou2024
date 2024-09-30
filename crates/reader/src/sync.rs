//! Blocking implementations of the reader.
//!
use crate::{config, utils};

use std::{
    fs,
    io::{self, BufRead, Read},
    ops::Deref,
};

// This is a new type that wraps a usize; this is only to enable the [`Default`] trait
pub struct ChunkSize(usize);

impl Default for ChunkSize {
    fn default() -> Self {
        Self(config::CHUNK_SIZE)
    }
}

impl From<usize> for ChunkSize {
    fn from(size: usize) -> Self {
        Self(size)
    }
}

impl Deref for ChunkSize {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ChunkSize> for usize {
    fn from(size: ChunkSize) -> Self {
        size.0
    }
}

/// Buffered file reader, reading the file in chunks.
///
/// This is a synchronous reader, and is used as a baseline for the performance of the
/// asynchronous reader. This is designed to be an [`Iterator`] over the chunks of slices of bytes.
pub struct FixedMemoryReader<R: io::Read, const ML: usize = { config::MAX_SENTENCE_LENGTH }> {
    inner: io::BufReader<R>,
    /// Internally, we use a usize to store the chunk size.
    pub chunk_size: usize,
    pub overflow: Vec<u8>,
    /// Pointer to where the buffer is currently writing to.
    pub overflow_pointer: usize,
}

impl<R: io::Read, const ML: usize> FixedMemoryReader<R, ML> {
    /// Create a new instance of the [`FixedMemoryReader`] using the provided
    /// [`io::BufReader`] instance.
    pub fn new(inner: io::BufReader<R>, chunk_size: impl Into<ChunkSize>) -> Self {
        let chunk_size: usize = chunk_size.into().into();
        Self {
            inner,
            chunk_size,
            overflow: utils::new_buffer(ML),
            overflow_pointer: 0,
        }
    }

    /// Create a new instance of the [`FixedMemoryReader`] using the provided
    /// object that implements [`io::Read`].
    pub fn from_read(reader: R, chunk_size: impl Into<ChunkSize>) -> Self {
        let inner = io::BufReader::with_capacity(config::CHUNK_SIZE, reader);

        Self::new(inner, chunk_size)
    }

    /// Read from the reader to try and fill the buffer, but only up to the last
    /// occurrence of the provided byte within the buffer size.
    pub fn take_until(&mut self, byte: u8, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.len() < self.chunk_size {
            panic!(
                "Buffer size ({buffer_size}) must be at least the chunk size ({chunk_size}).",
                buffer_size = buffer.len(),
                chunk_size = self.chunk_size,
            );
        }
        // We may already have some bytes in the internal buffer, so we need to
        // make sure we count them in the total length to read.
        let mut read = 0;

        // If we have some overflow from the last read, we need to copy it to the buffer.
        buffer[..self.overflow_pointer].copy_from_slice(&self.overflow[..self.overflow_pointer]);
        read += self.overflow_pointer;
        self.overflow.clear();
        self.overflow_pointer = 0;

        // Read a chunk of bytes regardless of separators.
        read += self
            .inner
            .read(&mut buffer[read..self.chunk_size.saturating_sub(ML)])?;

        // Now we try to find the last occurrence of the separator in the buffer.
        loop {
            let overflow_pointer = self.inner.read_until(byte, &mut self.overflow)?;

            if read + overflow_pointer >= self.chunk_size || overflow_pointer == 0 {
                self.overflow_pointer = overflow_pointer;
                return Ok(read);
            }

            // If our buffer is not full, we can copy the overflow to the buffer.
            buffer[read..read + overflow_pointer]
                .copy_from_slice(&self.overflow[..overflow_pointer]);
            self.overflow.clear();
            read += overflow_pointer;
        }
    }

    /// Iterate over the chunks of bytes in the memory-mapped file.
    pub fn iter<const SEP: u8>(&mut self) -> IterFixedMemoryReader<'_, SEP, R, ML> {
        IterFixedMemoryReader { reader: self }
    }
}

impl<const ML: usize> FixedMemoryReader<fs::File, ML> {
    /// Read the provided [`fs::File`] using [`FixedMemoryReader`].
    pub fn from_file(file: fs::File, chunk_size: impl Into<ChunkSize>) -> Self {
        Self::from_read(file, chunk_size)
    }

    /// Read the file at the given path using [`FixedMemoryReader`].
    pub fn from_path(
        path: impl AsRef<std::path::Path>,
        chunk_size: impl Into<ChunkSize>,
    ) -> Result<Self, io::Error> {
        std::fs::File::open(path).map(|file| Self::from_file(file, chunk_size))
    }

    /// Size of the file in bytes.
    pub fn size(&self) -> Result<usize, io::Error> {
        self.inner.get_ref().metadata().map(|f| f.len() as usize)
    }
}

/// An iterator over the chunks of bytes in the reader.
pub struct IterFixedMemoryReader<'m, const SEP: u8, R: io::Read, const ML: usize> {
    reader: &'m mut FixedMemoryReader<R, ML>,
}

impl<'m, const SEP: u8, R: io::Read, const ML: usize> Iterator
    for IterFixedMemoryReader<'m, SEP, R, ML>
{
    type Item = Vec<u8>;

    /// Read the next chunk of bytes from the reader.
    ///
    /// This is a simple wrapper around the [`FixedMemoryReader::take_until`] method, however
    /// this is less efficient as it creates a new buffer for each chunk.
    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = utils::new_buffer(self.reader.chunk_size);

        let bytes_read = self
            .reader
            .take_until(SEP, &mut buffer)
            .expect("Failed to read from the internal reader.");

        (bytes_read > 0).then_some(buffer)
    }
}
