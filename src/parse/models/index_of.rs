//! Index generation functions.
//!

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

static CHAR_MAP: OnceLock<HashMap<char, char>> = OnceLock::new();

macro_rules! create_index {
    ($($from:literal => $to:literal),*$(,)?) => {
        {
            let mut index = HashMap::new();
            $(index.insert($from, $to);)*
            index
        }
    };
}

/// Get the character mapping.
///
/// If the mapping has not been created, it will be created.
pub fn get_mapping() -> &'static HashMap<char, char> {
    CHAR_MAP.get_or_init(|| {
        create_index! {
            '4' => 'a',
            '8' => 'b',
            '3' => 'e',
            '9' => 'g',
            '1' => 'i',
            '1' => 'l',
            '0' => 'o',
            '5' => 's',
            '7' => 't',
            '2' => 'z',
            '®' => 'r',
        }
    })
}

/// Generate an index for the given item.
pub fn indices_of<const LENGTH: usize, const DEPTH: usize>(item: &str) -> IndexOf<LENGTH, DEPTH> {
    IndexOf::from(item)
}

/// An iterator over the indices of a string.
pub struct IndexOf<const LENGTH: usize, const DEPTH: usize = 1> {
    item: String,
    index: usize,
    seen: HashSet<String>,
}

impl<const LENGTH: usize, const DEPTH: usize> IndexOf<LENGTH, DEPTH> {
    /// Get the item of this index.
    pub fn item(&self) -> &str {
        &self.item
    }
}

/// Enables the conversion of a string to an index.
impl<const LENGTH: usize, const DEPTH: usize> From<&str> for IndexOf<LENGTH, DEPTH> {
    fn from(value: &str) -> Self {
        let mapping = get_mapping();
        Self {
            item: value
                .to_ascii_lowercase()
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(|c| *mapping.get(&c).unwrap_or(&c))
                .collect(),
            index: 0,
            seen: HashSet::new(),
        }
    }
}

/// Enables the iteration over the indices.
impl<const LENGTH: usize, const DEPTH: usize> Iterator for IndexOf<LENGTH, DEPTH> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // We have to safe guard this because otherwise it will attempt to create
        // an index at least once.
        if self.item.is_empty() {
            return None;
        }

        if
        // Currently only supports the beginning of the string.
        self.index > 0 && self.index + LENGTH > self.item.len() {
            return None;
        }

        let index = self.index;

        if index >= DEPTH {
            return None;
        }
        self.index += 1;

        let result = &self.item[index..(index + LENGTH).min(self.item.len())];

        // Prevents duplicates.
        if self.seen.contains(result) {
            return self.next();
        }

        self.seen.insert(result.to_owned());

        // Prevents index overflow.
        Some(result.to_owned())
    }
}

#[cfg(test)]
mod test {
    use std::array;

    use super::*;

    macro_rules! test_expand_indices {
        (
            $(($name:ident: $text:literal[$length:literal, $depth:literal] => $index:expr, $results:expr)),*$(,)?
        ) => {
            $(
                #[test]
                fn $name() {
                    let indices = indices_of::<$length, $depth>($text);
                    assert_eq!(indices.item(), $index);
                    let actual: Vec<_> = indices.collect();
                    assert_eq!(&actual, &$results);
                }
            )*
        };
    }

    test_expand_indices!(
        (empty_string: ""[3, 3] => "", array::from_fn::<&'static str, 0, _>(|_| "")),
        (simple_3l_1d: "P45sw0®D"[3, 1] => "password", ["pas"]),
        (simple_3l_3d: "P45sw0®D"[3, 3] => "password", ["pas", "ass", "ssw"]),
        (length_exceeds_item: "P45sw0®D"[9, 1] => "password", ["password"]),
        (depth_exceeds_item: "ABCDEFG"[1, 8] => "abcdefg", ["a", "b", "c", "d", "e", "f", "g"]),
        (duplicates: "P45sw0®D"[1, 8] => "password", ["p", "a", "s", "w", "o", "r", "d"]),
        (whitespaces: "P45s w0®D"[3, 3] => "password", ["pas", "ass", "ssw"])
    );
}
