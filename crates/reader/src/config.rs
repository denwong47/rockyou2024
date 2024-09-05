//! Configuration for the reader.

pub const CHUNK_SIZE: usize = 65536 * 8; // Max buffer capacity 2097152 - higher does not change anything.

/// The maximum sentence length, in bytes.
///
/// When reading a file, the reader will read up to [`CHUNK_SIZE`] minus this value to ensure
/// that the sentence is not split across chunks. It will then scan for the next separator
/// byte, and repeat until the chunk size is exceeded.
///
/// Underspecifying this value can cause sentences to be split across chunks.
pub const MAX_SENTENCE_LENGTH: usize = 1024;
