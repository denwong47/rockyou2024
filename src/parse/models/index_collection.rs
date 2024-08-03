//! A collection of indices.
//!

use fxhash::FxHashMap as HashMap;
use std::{io, path, sync::RwLock};

use super::indices_of;
use super::IndexFile;

/// A collection of indices.
pub struct IndexCollection<const LENGTH: usize, const DEPTH: usize, const MAX_SIZE: usize> {
    dir: path::PathBuf,
    indices: RwLock<HashMap<String, IndexFile>>,
}

impl<const LENGTH: usize, const DEPTH: usize, const MAX_SIZE: usize>
    IndexCollection<LENGTH, DEPTH, MAX_SIZE>
{
    /// Create a new index collection.
    pub fn new(dir: path::PathBuf) -> Self {
        Self {
            dir,
            indices: HashMap::default().into(),
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
}
