/// Get the indices of the given input.
///
/// For use in Go.
///
/// # ``LENGTH`` and ``DEPTH``
///
/// The length and depth of the index always default to the values in the configuration,
/// since the Rust FFI does not support generics.
char **indices_of(const char *input);

/// Clean the string using the specified search style.
char *as_search_string(
    const char *query,
    const char *search_style
);

/// Find the lines in the index collection.
///
/// This function is a wrapper around the [`IndexCollection::find_lines_containing`] method, which
/// does not report errors. This function will log any errors and return a null pointer if an error
/// occurs, including:
///
/// - The `dir` pointer is null.
/// - The path given by `dir` is not a directory.
/// - The `query` pointer is null.
/// - The `search_style` pointer is null.
/// - The `search_style` is not one of "strict", "case-insensitive", or "fuzzy".
///
/// For use in Go.
char **find_lines_in_index_collection(
    const char *dir,
    const char *query,
    const char *search_style
);
