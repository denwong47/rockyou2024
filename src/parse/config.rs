/// The path to the source file.
pub const SOURCE_PATH: &str = "data/raw/rockyou.csv";

/// The path to the output directory.
pub const INDEX_PATH: &str = "data/index";

/// The default number of threads.
pub const NUMBER_OF_THREADS: usize = 8;

/// The default chunk size.
pub const CHUNK_SIZE: usize = 65536;

/// The default maximum chunk size.
pub const MAX_CHUNK_SIZE: usize = 1048576;

/// The default number of characters to use as index.
pub const INDEX_LENGTH: usize = 3;

/// The depth of characters to use as index for each string.
pub const INDEX_DEPTH: usize = 1;

/// The maximum buffer size for each index file.
pub const MAX_INDEX_BUFFER_SIZE: usize = 2_usize.pow(12);

/// The default maximum buffer size for each index file.
pub const DEFAULT_MAX_BUFFER: usize = crate::config::MAX_INDEX_BUFFER_SIZE;

#[cfg(test)]
#[cfg(not(feature = "skip_index_write"))]
pub(crate) const TEST_DIR: &str = "./.tests";

#[cfg(test)]
pub(crate) const TEST_MOCK_INDEX: &str = "./.tests/mock_index";
