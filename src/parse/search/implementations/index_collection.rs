//! Adding search related methods to the index collection.
//!

use super::super::SearchStyle;
use crate::{
    models::{indices_of, IndexCollection, IndexFile},
    path_for_key,
};
use hashbrown::HashSet;
use rayon::prelude::*;
use std::io;

const LOG_TARGET: &str = "IndexCollection::search_for";

impl<const LENGTH: usize, const DEPTH: usize> IndexCollection<LENGTH, DEPTH> {
    /// Search for a string in the index.
    ///
    /// This will return a list of index files where the string could be found.
    pub fn index_files_for(&self, query: &str) -> Vec<IndexFile> {
        indices_of::<{ LENGTH }, { DEPTH }>(query.as_bytes())
            .map(|key| {
                // We could cache the index files, but that would create all sorts of race conditions.
                // Instead, we'll just create them on the fly.
                // Since we are just searching for the index, performance should not be a concern.
                (
                    key.clone(),
                    path_for_key(&key, &self.dir).and_then(IndexFile::from_path),
                )
            })
            .filter_map(|(key, result)| match result {
                Ok(index) => Some(index),
                Err(error) => {
                    crate::debug!(
                        target: LOG_TARGET,
                        "No index for {key:?}, error: {error:?}",
                        error = error
                    );
                    None
                }
            })
            .collect()
    }

    /// Search for a query in the whole index collection.
    pub fn find_lines_containing(&self, query: &str, search_style: SearchStyle) -> HashSet<String> {
        crate::debug!(
            target: LOG_TARGET,
            "Searching for {query:?} in the index collection...",
            query = query
        );

        let index_files = self.index_files_for(query);

        let chunks_count = usize::max(1, usize::min(index_files.len(), rayon::max_num_threads()));

        index_files
            .par_chunks(chunks_count)
            .map(|index| {
                index.iter().try_fold(HashSet::new(), |mut acc, index| {
                    crate::debug!(
                        target: LOG_TARGET,
                        "Searching for {query:?} in {index:?}",
                        query = query,
                        index = index.key
                    );

                    let acc_len = acc.len();

                    index
                        .find_lines_containing(&[query], search_style)?
                        .filter_map(
                            // We are only interested in the lines that are okay.
                            |line| line.ok(),
                        )
                        .for_each(|line| {
                            acc.insert(line);
                        });

                    crate::debug!(
                        target: LOG_TARGET,
                        "Found {count} lines for {query:?} in {index:?}.",
                        count = acc.len() - acc_len,
                        query = query,
                        index = index.key
                    );

                    Ok(acc)
                })
            })
            .filter_map(|result: io::Result<HashSet<String>>| result.ok())
            .reduce(HashSet::new, |acc, set| acc.union(&set).cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::TEST_MOCK_INDEX;
    use std::path;

    macro_rules! create_test_index_files_for {
        ($name:ident($query:expr) == $expected:expr) => {
            #[test]
            fn $name() {
                let expected = $expected.into_iter().map(ToString::to_string).collect::<HashSet<_>>();
                let path = path::PathBuf::from(TEST_MOCK_INDEX);
                if !path.is_dir() {
                    panic!(
                        "The test directory at {path:?} does not exist; please confirm that you have cloned the repository correctly.",
                        path = path.as_os_str()
                    );
                }
                let index = IndexCollection::<3, 1>::new(path);
                let actual = index.index_files_for($query).into_iter().map(
                    |index| index.key.clone()
                ).collect::<HashSet<_>>();

                assert_eq!(expected, actual);
            }
        };
    }

    create_test_index_files_for!(query_with_2_results("password") == ["pas", "wor"]);
    create_test_index_files_for!(query_by_prefix("conference") == ["con"]);
    create_test_index_files_for!(query_by_common_word("defcon") == ["con"]);

    macro_rules! create_search_test {
        ($name:ident<$length:literal, $depth:literal>($query:expr, $search_style:expr) == $expected:expr) => {
            #[test]
            fn $name() {
                let path = path::PathBuf::from(TEST_MOCK_INDEX);
                let index = IndexCollection::<$length, $depth>::new(path);
                let actual = index.find_lines_containing($query, $search_style);

                let expected = $expected
                    .into_iter()
                    .map(ToString::to_string)
                    .collect::<HashSet<_>>();

                assert_eq!(expected, actual);
            }
        };
    }

    create_search_test!(
        non_fuzzy_search<3, 1>("password", SearchStyle::Strict) == [
            "mypassword",
            "mapassword",
            "password13",
            "passwordz",
            "password5",
            "password75",
            "password1992",
            "password12",
            "password",
            "password1994",
            "password1!",
            "$password$",
            "password2",
            "!password!",
            "password123",
            "passwords",
            "xpasswordx",
            "password4",
            "(password)",
            "password3",
            "password.",
            "0password0",
            "**password**",
            "password11",
            "1password",
            "password7",
            "password!",
            "thisispassword",
            "password1",
            "thispassword",
        ]
    );
}
