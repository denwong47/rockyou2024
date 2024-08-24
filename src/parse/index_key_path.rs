use regex::Regex;
use std::{io, path, sync::OnceLock};

/// The prefix for index files.
const INDEX_PREFIX: &str = "subset_";

/// The suffix for index files.
const INDEX_EXTENSION: &str = "csv";

/// Returns the path for the given key and path.
pub fn path_for_key(
    key: impl AsRef<str>,
    dir: impl AsRef<path::Path>,
) -> io::Result<path::PathBuf> {
    let mut path_buf = path::Path::new(dir.as_ref()).to_path_buf();

    // There is no need to check if the directory exists in tests, since many of them don't.
    #[cfg(not(test))]
    if !path_buf.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "The directory does not exist at {path:?}",
                path = path_buf.as_os_str()
            ),
        ));
    }
    let file_name = format!("{}{}.{}", INDEX_PREFIX, key.as_ref(), INDEX_EXTENSION);

    path_buf.push(file_name);
    Ok(path_buf)
}

#[cfg(feature = "search")]
static FILE_NAME_PATTERN: OnceLock<Regex> = OnceLock::new();

#[cfg(feature = "search")]
fn file_name_pattern() -> &'static Regex {
    FILE_NAME_PATTERN.get_or_init(|| {
        Regex::new(&format!(
            r"^{}(?P<key>\S+)\.{}$",
            INDEX_PREFIX, INDEX_EXTENSION
        ))
        .unwrap()
    })
}

/// Returns the key for the given path if it is an index file; otherwise, returns `None`.
///
/// This does not assert that the file exists; it only checks the file name.
#[cfg(feature = "search")]
pub fn key_for_path(path: impl AsRef<path::Path>) -> Option<String> {
    let path = path.as_ref();
    let file_name = path.file_name()?.to_str()?;
    let re = file_name_pattern();
    let captures = re.captures(file_name)?;
    captures.get(1).map(|m| m.as_str().to_string())
}

#[cfg(feature = "search")]
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! create_test_reverse {
        ($name:ident($key:expr)) => {
            #[test]
            fn $name() {
                let key = $key;
                let dir = "/path/to/dir";

                let path =
                    path_for_key(key, dir).expect(&format!("Failed to get path for key {key:?}."));
                let extracted_key =
                    key_for_path(&path).expect(&format!("Failed to get key for path {path:?}."));

                assert_eq!(key, extracted_key);
            }
        };
    }

    create_test_reverse!(reverse_3char("abc"));
    create_test_reverse!(reverse_4char("ABCD"));
    create_test_reverse!(reverse_2char_with_number("a1b"));
    create_test_reverse!(reverse_2char_with_special("a_b"));

    macro_rules! create_test_valid {
        ($name:ident($path:expr) == $key:expr) => {
            #[test]
            fn $name() {
                let path_str = $path;
                let path = path::Path::new(&path_str);
                let extracted_key =
                    key_for_path(&path).expect(&format!("Failed to get key for path {path:?}."));

                assert_eq!($key, extracted_key);
            }
        };
    }

    create_test_valid!(valid_3char(format!("{INDEX_PREFIX}{}.{INDEX_EXTENSION}", "abc")) == "abc");
    create_test_valid!(
        valid_2char_with_number(format!("{INDEX_PREFIX}{}.{INDEX_EXTENSION}", "a1b")) == "a1b"
    );

    macro_rules! create_test_invalid {
        ($name:ident($path:expr)) => {
            #[test]
            fn $name() {
                let path_str = $path;
                let path = path::Path::new(&path_str);
                let extracted_key = key_for_path(&path);

                assert!(extracted_key.is_none());
            }
        };
    }

    create_test_invalid!(invalid_empty(""));
    create_test_invalid!(invalid_no_extension(format!("{INDEX_PREFIX}{}", "abc")));
    create_test_invalid!(invalid_no_prefix(format!("{}{}", "abc", INDEX_EXTENSION)));
    create_test_invalid!(invalid_no_key(format!(
        "{}.{}",
        INDEX_PREFIX, INDEX_EXTENSION
    )));
}
