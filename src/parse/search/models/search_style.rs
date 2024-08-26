//! The style of search to perform.
//!
//!
use std::io::Read;

use super::ManipulatedReader;
use crate::string;

/// The style of search to perform.
#[derive(Debug, Clone, Copy)]
pub enum SearchStyle {
    Strict,
    CaseInsensitive,
    Fuzzy,
}

impl SearchStyle {
    /// Transform a query string into the desired format.
    pub fn transform_query<'s>(&self) -> fn(&[&'s str]) -> Vec<String> {
        match self {
            SearchStyle::Strict => |s| s.iter().map(|s| s.to_string()).collect(),
            SearchStyle::CaseInsensitive => |s| s.iter().map(|s| s.to_ascii_lowercase()).collect(),
            SearchStyle::Fuzzy => |s| {
                s.iter()
                    .map(|s| string::convert_to_fuzzy_string(s).collect::<String>())
                    .collect()
            },
        }
    }

    /// Get the query style from a string.
    pub fn transform_reader<R: Read + 'static>(&self, reader: R) -> Box<dyn Read> {
        // Is this efficient?
        match self {
            SearchStyle::Strict => Box::new(reader),
            SearchStyle::CaseInsensitive => Box::new(ManipulatedReader::new(reader, |buffer| {
                buffer.to_ascii_lowercase()
            })),
            SearchStyle::Fuzzy => Box::new(ManipulatedReader::new(reader, |buffer| {
                string::convert_to_fuzzy_string(&String::from_utf8_lossy(buffer))
                    .collect::<String>()
                    .as_bytes()
                    .to_vec()
            })),
        }
    }
}
