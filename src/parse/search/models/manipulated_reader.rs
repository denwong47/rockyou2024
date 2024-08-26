//! A [`std::io::Read`] implementation that wraps another [`std::io::Read`]
//! implementation, and mutates the bytes read from the inner reader.
//!
//! This is useful for cases where you want to read from a reader, but also
//! want to modify the bytes read in some way, such as case-insensitive
//! searching.

use std::io::{self, Read};

/// A [`std::io::Read`] implementation that wraps another [`std::io::Read`]
/// implementation, and mutates the bytes read from the inner reader.
///
/// This is useful for cases where you want to read from a reader, but also
/// want to modify the bytes read in some way, such as case-insensitive
/// searching.
pub struct ManipulatedReader<R>
where
    R: Read,
{
    inner: R,
    manipulator: fn(&[u8]) -> Vec<u8>,
}

impl<R> ManipulatedReader<R>
where
    R: Read,
{
    /// Create a new [`ManipulatedReader`] from an inner reader and a manipulator.
    pub fn new(inner: R, manipulator: fn(&[u8]) -> Vec<u8>) -> Self {
        Self { inner, manipulator }
    }
}

impl<R> Read for ManipulatedReader<R>
where
    R: Read,
{
    /// Read bytes from the inner reader, and manipulate them.
    ///
    /// If the manipulation results in a different length than the original bytes,
    /// it will be treated as though the manipulated bytes were from the inner reader;
    /// the buffer will be changed up to the new length.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut inner_buf = vec![0; buf.len()];
        let bytes_read = self.inner.read(inner_buf.as_mut_slice())?;
        let manipulated = (self.manipulator)(&inner_buf[..bytes_read]);
        buf[..manipulated.len()].copy_from_slice(&manipulated);
        Ok(manipulated.len())
    }
}
