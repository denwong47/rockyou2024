//! Common functions for strings.
//!
use unicode_normalization::UnicodeNormalization;

/// Convert an extended string to ASCII.
///
/// Decompse the input string into its base characters and filter out combining diacritical marks.
pub fn convert_extended_to_ascii(input: &str) -> impl Iterator<Item = char> + '_ {
    input
        .nfd() // Perform Unicode Normalization Form D (decomposition)
        .filter(|c| !matches!(*c, '\u{0300}'..='\u{036f}')) // Filter out combining diacritical marks
}
