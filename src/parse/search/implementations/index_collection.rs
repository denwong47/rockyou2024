//! Adding search related methods to the index collection.
//!

use super::super::SearchStyle;
use crate::{
    models::{indices_of, IndexCollection, IndexCollectionResult, IndexFile},
    path_for_key,
};
use hashbrown::HashSet;
use rayon::prelude::*;
use std::io;

#[cfg(feature = "lru")]
use std::sync::{Arc, RwLock, RwLockWriteGuard};

#[cfg(feature = "lru")]
use crate::models::IndexCollectionCache;

#[cfg(feature = "lru")]
use crate::config::CACHE_SIZE;

const LOG_TARGET: &str = "IndexCollection::search_for";

#[cfg(feature = "lru")]
pub type IndexCollectionReturn = Arc<IndexCollectionResult>;

#[cfg(not(feature = "lru"))]
pub type IndexCollectionReturn = IndexCollectionResult;

#[cfg(feature = "lru")]
fn reset_cache_on_poisoned(
    cache: &RwLock<IndexCollectionCache>,
    err: &mut std::sync::PoisonError<RwLockWriteGuard<'_, IndexCollectionCache>>,
) -> Option<Arc<IndexCollectionResult>> {
    crate::error!(
        target: LOG_TARGET,
        "Failed to acquire lock on cache; cache might be poisoned: {err:?}. Resetting cache...",
        err = err
    );
    **err.get_mut() = lru::LruCache::new(std::num::NonZeroUsize::new(CACHE_SIZE).expect(
        "Failed to create a non-zero usize from the cache size; this should be unreachable.",
    ));
    cache.clear_poison();

    // Cache is now empty, so we can just return None.
    None
}

impl<const LENGTH: usize, const DEPTH: usize, const MAX_BUFFER: usize>
    IndexCollection<LENGTH, DEPTH, MAX_BUFFER>
{
    /// Search for a string in the index.
    ///
    /// This will return a list of index files where the string could be found.
    pub fn index_files_for(&self, query: &str) -> Vec<IndexFile<MAX_BUFFER>> {
        indices_of::<{ LENGTH }, { DEPTH }>(query.as_bytes())
            .map(|key| {
                // We could cache the index files, but that would create all sorts of race conditions.
                // Instead, we'll just create them on the fly.
                // Since we are just searching for the index, performance should not be a concern.
                (
                    key.clone(),
                    path_for_key(&key, &self.dir).and_then(IndexFile::<{ MAX_BUFFER }>::from_path),
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
    pub fn find_lines_containing(
        &self,
        query: &str,
        search_style: SearchStyle,
    ) -> IndexCollectionReturn {
        #[cfg(feature = "lru")]
        {
            if let Some(cache_hit) = self
                .cache
                .write()
                .map(|mut cache| cache.get(query).cloned())
                .unwrap_or_else(
                    // A cache is just a cache; if it's poisoned, we'll just reset it.
                    |mut err| reset_cache_on_poisoned(&self.cache, &mut err),
                )
            {
                crate::debug!(
                    target: LOG_TARGET,
                    "Cache hit for {query:?} in the index collection, returning {count} cached result.",
                    query = query,
                    count = cache_hit.len()
                );
                return cache_hit;
            }

            crate::debug!(
                target: LOG_TARGET,
                "Cache miss for {query:?} in the index collection.",
                query = query
            );
        }

        crate::debug!(
            target: LOG_TARGET,
            "Searching for {query:?} in the index collection...",
            query = query
        );

        let index_files = self.index_files_for(query);

        let chunks_count = usize::max(1, usize::min(index_files.len(), rayon::max_num_threads()));

        let results: IndexCollectionReturn = index_files
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
            .into(); // Convert to an Arc.

        #[cfg(feature = "lru")]
        {
            crate::debug!(
                target: LOG_TARGET,
                "Caching the {count} results found for key {query:?}.",
                count = results.len(),
                query = query
            );
            self.cache
                .write()
                .map(|mut cache| {
                    cache.put(query.to_owned(), Arc::clone(&results));
                })
                .unwrap_or_else(
                    // A cache is just a cache; if it's poisoned, we'll just reset it.
                    |mut err| {
                        reset_cache_on_poisoned(&self.cache, &mut err);
                    },
                );
        }

        #[cfg(not(feature = "lru"))]
        {
            crate::warn!(
                target: LOG_TARGET,
                "The search cache is disabled; not caching the {count} results found for key {query:?}.",
                count = results.len(),
                query = query
            );
        }

        results
    }

    /// Search for a query in the whole index collection.
    ///
    /// This method will return a paginated list of results; the offset and limit
    /// parameters are used to determine which results to return.
    ///
    /// Contrary to `find_lines_containing`, this method will return an owned
    /// `HashSet` of strings, instead of an `Arc`, since the results won't be reused.
    ///
    /// # Note
    ///
    /// Since the Lru cache does not persist between calls to the FFI functions,
    /// this method is not available in the FFI; and is only meaningful when
    /// the `lru` feature is enabled.
    pub fn find_lines_containing_paginated(
        &self,
        query: &str,
        search_style: SearchStyle,
        offset: usize,
        limit: usize,
    ) -> HashSet<String> {
        self.find_lines_containing(query, search_style)
            .iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect()
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

                #[cfg(feature = "lru")]
                assert_eq!(&expected, actual.as_ref());

                #[cfg(not(feature = "lru"))]
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
