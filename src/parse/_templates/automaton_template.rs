/// This generates a cached automaton for `{{LIST_NAME}}`.
use aho_corasick::AhoCorasick;

#[cfg(not(test))]
use std::sync::OnceLock;

/// The automaton.
#[cfg(not(test))]
static AUTOMATON: OnceLock<(usize, AhoCorasick)> = OnceLock::new();

/// Initialize the automaton.
#[allow(dead_code)]
fn init_automaton<const LENGTH: usize>() -> AhoCorasick {
    let patterns = &["{{WORD_LIST}}"];

    AhoCorasick::new(patterns.iter().filter_map(|word| match word.len() {
        l if l < LENGTH => None,
        l if l == LENGTH => Some(*word),
        _ => word.get(..LENGTH),
    }))
    .expect("Failed to create automaton for '{{LIST_NAME}}'. Please check the word list.")
}

/// Get the automaton.
///
/// Only one automaton length is allowed per execution. As `LENGTH` is a `const`, this is
/// enforced at compile-time.
///
/// However this prevents unit tests from running in parallel, as they may request different automaton
/// lengths.
#[allow(dead_code)]
#[cfg(not(test))]
pub fn get_automaton<const LENGTH: usize>() -> &'static AhoCorasick {
    let (length, automaton) = AUTOMATON.get_or_init(|| (LENGTH, init_automaton::<LENGTH>()));

    if length != &LENGTH {
        panic!("Automaton for '{{LIST_NAME}}' has already been initialized with a different length of {existing}, but was requested with a length of {requested}.", existing = length, requested = LENGTH);
    } else {
        automaton
    }
}

/// Get the automaton.
///
/// For unit tests, do not store the automaton in a static variable.
#[allow(dead_code)]
#[cfg(test)]
pub fn get_automaton<const LENGTH: usize>() -> AhoCorasick {
    init_automaton::<LENGTH>()
}
