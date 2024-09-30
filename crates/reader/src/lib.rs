//! The reader coroutine.

pub mod config;
mod sync;

pub mod utils;

pub use sync::{ChunkSize, FixedMemoryReader, IterFixedMemoryReader};
