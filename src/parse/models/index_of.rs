//! Index generation functions.
//!

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::OnceLock;

use crate::{automatons, string};

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
            '6' => 'g',
            '9' => 'g',
            '1' => 'i',
            'l' => 'i',
            '0' => 'o',
            '5' => 's',
            '7' => 't',
            '2' => 'z',
            '®' => 'r',
            '£' => 'e',
            '€' => 'e',
            '$' => 's',
            '@' => 'a',
            '!' => 'i',
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
    matches: VecDeque<aho_corasick::Match>,
    seen: HashSet<String>,
}

impl<const LENGTH: usize, const DEPTH: usize> IndexOf<LENGTH, DEPTH> {
    /// Get the item of this index.
    pub fn item(&self) -> &str {
        &self.item
    }

    /// Get index by position.
    pub fn next_by_position(&mut self) -> Option<String> {
        // We have to safe guard this because otherwise it will attempt to create
        // an index at least once.
        if self.item.is_empty() {
            return None;
        }

        // Currently only supports the beginning of the string.
        if self.index + LENGTH > self.item.len() {
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

    /// Get index by common english words.
    pub fn next_by_common_words(&mut self) -> Option<String> {
        self.matches.pop_front().and_then(|matched| {
            let word = self.item.get(matched.start()..matched.end())?;

            if !self.seen.insert(word.to_owned()) {
                self.next_by_common_words()
            } else {
                Some(word.to_owned())
            }
        })
    }
}

/// Enables the conversion of a string to an index.
impl<const LENGTH: usize, const DEPTH: usize> From<&str> for IndexOf<LENGTH, DEPTH> {
    fn from(value: &str) -> Self {
        let mapping = get_mapping();
        let cleaned = string::convert_extended_to_ascii(&value.to_ascii_lowercase())
            .map(|c| *mapping.get(&c).unwrap_or(&c))
            .filter(|c| c.is_ascii_alphanumeric())
            .collect();
        let matches = automatons::en_common_words::get_automaton::<LENGTH>()
            .find_iter(&cleaned)
            .collect();

        Self {
            item: cleaned,
            index: 0,
            matches,
            seen: HashSet::new(),
        }
    }
}

/// Enables the iteration over the indices.
impl<const LENGTH: usize, const DEPTH: usize> Iterator for IndexOf<LENGTH, DEPTH> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_by_position()
            .or_else(|| self.next_by_common_words())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_expand_indices {
        (
            $(($name:ident: $method:ident<$length:literal, $depth:literal>($text:literal) => $index:expr, $results:expr)),*$(,)?
        ) => {
            $(
                #[test]
                fn $name() {
                    let mut indices = indices_of::<$length, $depth>($text);
                    assert_eq!(indices.item(), $index);
                    let actual: Vec<_> = {
                        let mut collector = Vec::new();
                        while let Some(index) = indices.$method() {
                            collector.push(index);
                        }
                        collector
                    };
                    assert_eq!(&actual, &$results);
                }
            )*
        };
    }

    static EMPTY_ARRAY: [&str; 0] = [];
    test_expand_indices!(
        (by_position_empty_string: next_by_position<3, 3>("") => "", EMPTY_ARRAY),
        (by_position_simple_3l_1d: next_by_position<3, 1>("P45sw0®D") => "password", ["pas"]),
        (by_position_simple_3l_3d: next_by_position<3, 3>("P45sw0®D") => "password", ["pas", "ass", "ssw"]),
        (by_position_length_exceeds_item: next_by_position<9, 1>("P45sw0®D") => "password", EMPTY_ARRAY),
        (by_position_depth_exceeds_item: next_by_position<1, 8>("ABCDEFG") => "abcdefg", ["a", "b", "c", "d", "e", "f", "g"]),
        (by_position_duplicates: next_by_position<1, 8>("P45sw0®D") => "password", ["p", "a", "s", "w", "o", "r", "d"]),
        (by_position_whitespaces: next_by_position<3, 3>("P45s w0®D") => "password", ["pas", "ass", "ssw"]),
        (by_common_words_empty_string: next_by_common_words<3, 3>("") => "", EMPTY_ARRAY),
        (by_common_words_simple_3l_1d: next_by_common_words<3, 1>("P45sw0®D") => "password", ["pas", "wor"]),
        // DEPTH has no effect on common words.
        (by_common_words_simple_3l_3d: next_by_common_words<3, 3>("P45sw0®D") => "password", ["pas", "wor"]),
        (by_common_words_simple_4l_1d: next_by_common_words<4, 1>("P45sw0®D") => "password", ["pass", "word"]),
    );

    #[test]
    fn combined_iterator() {
        let indices = indices_of::<4, 2>("My P45sw0®D").collect::<Vec<_>>();

        assert_eq!(indices, vec!["mypa", "ypas", "pass", "word"])
    }
}
