//! A collection of indices.
//!

use hashbrown::HashMap;
use std::{io, ops::DerefMut, path, sync::RwLock};

use super::{indices_of, IndexFile};
use crate::config::DEFAULT_MAX_BUFFER;

#[cfg(feature = "search")]
pub type IndexCollectionResult = hashbrown::HashSet<String>;

#[cfg(feature = "search_lru")]
/// Alias for the cache of the index collection.
pub type IndexCollectionCache = lru::LruCache<String, Arc<IndexCollectionResult>>;

#[cfg(feature = "search_lru")]
pub use std::sync::Arc;

#[cfg(feature = "search_lru")]
pub use crate::config::CACHE_SIZE;

/// A collection of indices.
pub struct IndexCollection<
    const LENGTH: usize,
    const DEPTH: usize,
    const MAX_BUFFER: usize = DEFAULT_MAX_BUFFER,
> {
    pub(crate) dir: path::PathBuf,
    pub(crate) indices: RwLock<HashMap<String, IndexFile<MAX_BUFFER>>>,

    #[cfg(feature = "search_lru")]
    /// A cache of the previous searches.
    pub(crate) cache: RwLock<IndexCollectionCache>,
}

impl<const LENGTH: usize, const DEPTH: usize, const MAX_BUFFER: usize>
    IndexCollection<LENGTH, DEPTH, MAX_BUFFER>
{
    /// Create a new index collection.
    pub fn new(dir: path::PathBuf) -> Self {
        Self {
            dir,
            indices: HashMap::default().into(),

            #[cfg(feature = "search_lru")]
            cache: RwLock::new(lru::LruCache::new(std::num::NonZeroUsize::new(CACHE_SIZE).expect(
                "Failed to create a non-zero usize from the cache size; this should be unreachable."
            ))),
        }
    }

    /// Add an item to the collection.
    pub fn add(&self, item: Vec<u8>) -> io::Result<()> {
        let mut indices = indices_of::<LENGTH, DEPTH>(&item);

        indices
        .try_for_each(
            |index| {
                self.assert_index_exists(&index)?;

                self.indices.read().expect(
                    "Failed to acquire read lock on indices; indices might be poisoned."
                ).get(&index).expect(
                    "Index does not exist in the collection after assertion, this should be unreachable."
                ).add(item.to_owned()).map(|_| ())
            }
        )
    }

    /// Add an index to the collection.
    fn assert_index_exists(&self, key: &str) -> io::Result<bool> {
        let mut indices = self
            .indices
            .write()
            .expect("Failed to acquire write lock on indices; indices might be poisoned.");
        if !indices.contains_key(key) {
            let index = IndexFile::new(key.to_owned(), &self.dir)?;
            Ok(indices.insert(key.to_owned(), index).is_none())
        } else {
            Ok(false)
        }
    }

    /// Post-process the collection.
    fn post_process(&mut self) -> io::Result<()> {
        let mut new_map = HashMap::default();

        // Swap the indices with a new map, so that we can consume the old map as we
        // post-process the indices.
        std::mem::swap(
            self.indices
                .write()
                .expect("Failed to acquire write lock on indices; indices might be poisoned.")
                .deref_mut(),
            &mut new_map,
        );

        new_map
            .into_iter()
            .try_for_each(|(_, mut index)| index.post_process())
    }
}

impl<const LENGTH: usize, const DEPTH: usize, const MAX_BUFFER: usize> Drop
    for IndexCollection<LENGTH, DEPTH, MAX_BUFFER>
{
    fn drop(&mut self) {
        self.post_process()
            .expect("Failed to post-process the index collection.");
    }
}
