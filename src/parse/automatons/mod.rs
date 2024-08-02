//! This module contains the automatons used to parse the input text.
//!
//! These are built during the build process and are used to parse the input text.

/// This module contains an aho-corasick automaton pre-loaded with 3,600
/// common English words.
pub mod en_common_words {
    include!(concat!(env!("OUT_DIR"), "/en_common_words.rs"));
    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn automaton_3() {
            let automaton = get_automaton::<3>();

            assert!(automaton.is_match("act"));
            assert!(automaton.is_match("you"));
            assert!(automaton.is_match("good acting"));

            // Short words are not matched.
            assert!(!automaton.is_match("  in "));

            // Long words are only matched by the first 3 characters.
            assert!(automaton.is_match("abl"));

            // Non-words are not matched.
            assert!(!automaton.is_match("zzz"));

            // The original long words should be matched.
            assert!(automaton.is_match("manufacturer"));

            let haystack = "The quick brown fox jumps over the lazy dog.".to_ascii_lowercase();
            assert_eq!(
                automaton
                    .find_iter(&haystack)
                    .map(|matched| haystack.get(matched.start()..matched.end()).unwrap())
                    .collect::<Vec<_>>(),
                vec!["the", "qui", "bro", "jum", "ove", "the", "laz", "dog",]
            );
        }

        #[test]
        fn automaton_4() {
            let automaton = get_automaton::<4>();

            assert!(!automaton.is_match("act"));
            assert!(automaton.is_match("able"));
            assert!(automaton.is_match("table"));
            let haystack = "The quick brown fox jumps over the lazy dog.".to_ascii_lowercase();
            assert_eq!(
                automaton
                    .find_iter(&haystack)
                    .map(|matched| haystack.get(matched.start()..matched.end()).unwrap())
                    .collect::<Vec<_>>(),
                vec!["quic", "brow", "jump", "over", "lazy"]
            );
        }
    }
}
