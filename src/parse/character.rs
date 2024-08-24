//! Utilities for working with characters.
//!

pub enum CharacterClass {
    Alphanumeric(char),
    Punctuation,
    Arabic,
    Chinese,
    Japanese,
    Korean,
    Unclassified,
}

impl From<char> for CharacterClass {
    fn from(c: char) -> Self {
        match c {
            c if c.is_ascii_alphanumeric() => Self::Alphanumeric(c),
            c if c.is_whitespace()
                || c.is_control()
                || c.is_ascii_punctuation()
                || ['-', '_', '(', ')'].contains(&c) =>
            {
                Self::Punctuation
            }
            c => {
                match c as u32 {
                    0x4E00..=0x9FFF |
                        0x3400..=0x4DBF |
                        0x20000..=0x2A6DF |
                        0x2A700..=0x2B73F |
                        0x2B740..=0x2B81F |
                        0x2B820..=0x2CEAF |
                        0x2CEB0..=0x2EBEF |
                        0x30000..=0x3134F |
                        0xF900..=0xFAFF |
                        0x2F800..=0x2FA1F
                        => Self::Chinese,
                    0xAC00..=0xD7AF | // Hangul Syllables
                        0x1100..=0x11FF | // Hangul Jamo
                        0x3130..=0x318F | // Hangul Compatibility Jamo
                        0xA960..=0xA97F | // Hangul Jamo Extended-A
                        0xD7B0..=0xD7FF   // Hangul Jamo Extended-B
                        => Self::Korean,
                    0x3040..=0x309F | // Hiragana
                        0x30A0..=0x30FF | // Katakana
                        0x31F0..=0x31FF | // Katakana Phonetic Extensions
                        // Shared with Chinese, cannot match
                        // 0x4E00..=0x9FFF | // CJK Unified Ideographs (Shared with Chinese)
                        // 0xF900..=0xFAFF | // CJK Compatibility Ideographs (Shared with Chinese)
                        0xFF00..=0xFFEF   // Halfwidth and Fullwidth Forms (including Katakana)
                        => Self::Japanese,
                    0x0600..=0x06FF | // Arabic
                        0x0750..=0x077F | // Arabic Supplement
                        0x08A0..=0x08FF | // Arabic Extended-A
                        0xFB50..=0xFDFF | // Arabic Presentation Forms-A
                        0xFE70..=0xFEFF | // Arabic Presentation Forms-B
                        0x1EE00..=0x1EEFF // Arabic Mathematical Alphabetic Symbols
                        => Self::Arabic,
                    _ => Self::Unclassified,
                }
            }
        }
    }
}

impl CharacterClass {
    /// Convert the character class to a substitution symbol, which is suitable for the
    /// file system.
    pub fn to_substitution_symbol(self) -> Option<char> {
        match self {
            Self::Alphanumeric(c) => Some(c),
            Self::Punctuation => None,
            Self::Chinese => Some('1'),
            Self::Japanese => Some('2'),
            Self::Korean => Some('3'),
            Self::Arabic => Some('4'),
            _ => None,
        }
    }
}
