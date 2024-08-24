/// Get the indices of the given input.
///
/// For use in Go.
///
/// # ``LENGTH`` and ``DEPTH``
///
/// The length and depth of the index always default to the values in the configuration,
/// since the Rust FFI does not support generics.
char **indices_of(const char *input);
