use std::{io, path};

use super::super::LinesScanner;
use crate::{key_for_path, models::IndexFile};

impl IndexFile {
    /// Create an [`IndexFile`] from an existing file.
    ///
    /// This typically is not for creating new files, but for reading existing files.
    pub fn from_path(path: impl AsRef<path::Path>) -> Result<Self, io::Error> {
        if !path.as_ref().is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{path:?} does not exist.", path = path.as_ref().as_os_str()),
            ));
        }

        let key = key_for_path(&path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "The file at {path:?} is not a valid index file.",
                    path = path.as_ref().as_os_str()
                ),
            )
        })?;

        Ok(Self {
            key,
            dir: path
                .as_ref()
                .parent()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "The file has no parent directory.",
                    )
                })?
                .to_path_buf(),
            seen: Default::default(),
            buffer: Default::default(),
        })
    }

    /// Read the index file, buffered.
    ///
    /// Different from [`IndexFile::open_for_write`], this function will not create
    /// the file if it does not exist.
    pub fn open_for_read(&self) -> Result<io::BufReader<std::fs::File>, io::Error> {
        std::fs::File::open(self.path()?).map(io::BufReader::new)
    }

    /// Search for some keys in the index file.
    pub fn find_lines_containing(
        &self,
        keys: &[&str],
    ) -> Result<LinesScanner<std::fs::File>, io::Error> {
        LinesScanner::new(|| self.open_for_read(), keys)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use crate::{config::TEST_MOCK_INDEX, index_key_path::path_for_key};

    /// This test relies on the test data being exactly as provided.
    #[test]
    fn non_fuzzy_search() {
        let path = path_for_key("pas", TEST_MOCK_INDEX)
            .expect("Failed to create a path for the key 'pas'.");
        let index = IndexFile::from_path(&path)
            .expect("The index file for 'pas' could not be found, or could not be read.");
        let scanner = index
            .find_lines_containing(&["password"])
            .expect("The scanner could not be created.");
        let lines = scanner
            .collect::<Result<HashSet<_>, _>>()
            .expect("An error occurred while reading the lines.");
        assert_eq!(
            lines,
            HashSet::from_iter(
                vec![
                    "password",
                    "password1",
                    "password2",
                    "password123",
                    "passwordz",
                    "password75",
                    "password1994",
                    "password1992",
                    "password1!",
                    "1password",
                    "0password0",
                    "password12",
                    "**password**",
                    "password3",
                    "mypassword",
                ]
                .into_iter()
                .map(ToString::to_string)
            )
        );
    }
}
