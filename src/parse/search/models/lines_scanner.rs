//! A wrapper around [`std::io::BufReader`] that can search for the line that contains
//! a given key.

use std::{
    io,
    io::{BufRead, BufReader, Read, Seek},
};

use aho_corasick::AhoCorasick;

use super::SearchStyle;

/// A scanner for searching for a key in an index.
///
/// This scanner will search for the key in the index and iterate over lines
/// that contain the key.
pub struct LinesScanner<R: Seek + Read + 'static> {
    reader: BufReader<R>,
    ranges: <Vec<aho_corasick::Match> as IntoIterator>::IntoIter,
}

/// The suspected maximum length of a line.
const MAX_LINE_LENGTH: usize = 256;

impl<R: Seek + Read + 'static> LinesScanner<R> {
    /// Create a new scanner.
    ///
    /// [`aho_corasick`] errors will be coerced into [`std::io::Error`].
    pub fn new(
        reader_factory: impl Fn() -> io::Result<BufReader<R>>,
        query: &[&str],
        search_style: SearchStyle,
    ) -> io::Result<Self> {
        let transformed_query = search_style.transform_query()(query);
        crate::debug!("Transformed query: {:?}", transformed_query);
        let ac = AhoCorasick::new(transformed_query).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to create Aho-Corasick automaton: {}", error),
            )
        })?;

        let ranges = ac
            .try_stream_find_iter(search_style.transform_reader(reader_factory()?))
            .map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "An error had occurred during an Aho-Corasick search; \
                    typically this is limited to some kind of misconfiguration \
                    that resulted in an illegal call: {error}"
                    ),
                )
            })?
            .map(|range| {
                range.map_err(|error| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "An error had occurred during an Aho-Corasick matching process; \
                        typically this is limited to some kind of misconfiguration \
                        that resulted in an illegal call: {error}"
                        ),
                    )
                })
            })
            .collect::<io::Result<Vec<_>>>()?;

        Ok(Self {
            reader: reader_factory()?,
            ranges: ranges.into_iter(),
        })
    }

    /// Find the line that contains the key.
    fn line_of_range(&mut self, range: aho_corasick::Match) -> io::Result<String> {
        let mut buffer = String::new();
        let mut pos = range.start().saturating_sub(MAX_LINE_LENGTH);

        while pos < range.end() {
            buffer.clear();
            self.reader.seek(io::SeekFrom::Start(pos as u64))?;
            pos += self.reader.read_line(&mut buffer)?;
        }

        let line = buffer.trim_end();

        if line.is_empty() {
            // This should not happen, but just in case.
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Got an empty line from the reader.",
            ));
        }

        Ok(line.to_owned())
    }
}

impl<R: Seek + Read> Iterator for LinesScanner<R> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ranges.next().map(|range| self.line_of_range(range))
    }
}