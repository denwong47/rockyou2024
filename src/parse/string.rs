//! Common functions for strings.
//!
use super::character;
use unicode_normalization::UnicodeNormalization;

/// Convert an extended string to ASCII.
///
/// Decompse the input string into its base characters and filter out combining diacritical marks.
pub fn convert_extended_to_ascii(input: &str) -> impl Iterator<Item = char> + '_ {
    input
        .nfd() // Perform Unicode Normalization Form D (decomposition)
        .filter(|c| !matches!(*c, '\u{0300}'..='\u{036f}')) // Filter out combining diacritical marks
}

/// Convert a string to a fuzzy string.
///
/// This function converts a string to a fuzzy string by converting it to lowercase and replacing
/// characters with similar characters, commonly used in passwords.
pub fn convert_to_fuzzy_string(input: &str) -> impl Iterator<Item = char> + '_ {
    map_characters_to_fuzzy(convert_extended_to_ascii(input))
}

/// Map a character to a similar character, in the password sense.
pub fn map_characters_to_fuzzy<'c>(
    chars: impl Iterator<Item = char> + 'c,
) -> impl Iterator<Item = char> + 'c {
    let mapping: &hashbrown::HashMap<char, char> = character::get_fuzzy_mapping();

    chars
        .map(|c| c.to_ascii_lowercase())
        .map(|c| *mapping.get(&c).unwrap_or(&c))
}
