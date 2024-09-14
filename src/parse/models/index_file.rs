use std::{
    fs,
    io::{self, Write},
    mem, path,
    sync::Mutex,
};

use crate::{config::DEFAULT_MAX_BUFFER, path_for_key};

#[cfg(feature = "deduplicate")]
use hashbrown::HashSet;

#[cfg(feature = "deduplicate")]
type FxHashSet32<T> = HashSet<T, std::hash::BuildHasherDefault<fxhash::FxHasher32>>;

/// A buffer for an index file, for a specific key.
///
/// Since the combined file is way too large to fit into memory, we have to "index"
/// the file into smaller files, each with a specific search key that each of the
/// contents of the file can be searched for.
///
/// The search key may or may not be contained within each of the items in the file;
/// this is just a way to limit the search space.
///
/// Typically there will be a mapping from search keys to their respective index files,
/// and the index files will be stored in a directory.
pub struct IndexFile<const MAX_BUFFER: usize = DEFAULT_MAX_BUFFER> {
    /// This is private because this should not be changed after creation.
    ///
    /// The key should only be accessible by the mapping that links to this
    /// [`IndexFile`].
    pub(crate) key: String,
    pub(crate) dir: path::PathBuf,

    #[cfg(feature = "deduplicate")]
    pub(crate) seen: Mutex<FxHashSet32<Vec<u8>>>,

    pub(crate) buffer: Mutex<Vec<u8>>,
}

impl<const MAX_BUFFER: usize> std::fmt::Debug for IndexFile<MAX_BUFFER> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexFile")
            .field("key", &self.key)
            .field("dir", &self.dir)
            .finish()
    }
}

/// The target for the crate.
const LOG_TARGET: &str = "IndexFile";

impl<const MAX_SIZE: usize> IndexFile<MAX_SIZE> {
    /// Creates a new instance of [`IndexFile`].
    pub fn new(key: String, dir: impl AsRef<path::Path>) -> io::Result<Self> {
        crate::debug!(
            target: LOG_TARGET,
            "Creating a new index for '{key}' at {dir:?}.",
            key=key,
            dir=dir.as_ref(),
        );

        #[cfg(feature = "deduplicate")]
        let seen = FxHashSet32::default().into();

        let buffer = Vec::with_capacity(DEFAULT_MAX_BUFFER).into();

        fs::create_dir_all(&dir)?;
        Ok(Self {
            key,
            dir: dir.as_ref().to_owned(),

            #[cfg(feature = "deduplicate")]
            seen,

            buffer,
        })
    }

    /// Returns the path for the index file.
    pub fn path(&self) -> io::Result<path::PathBuf> {
        path_for_key(&self.key, &self.dir)
    }

    /// Open the file associated with the key.
    pub fn open_for_write(&self) -> io::Result<fs::File> {
        fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.path()?)
    }

    /// Dispose of the existing index if it exists.
    ///
    /// This does not remove the index from the current buffer; this only removes the file.
    pub fn dispose(&self) -> io::Result<()> {
        let path = path_for_key(&self.key, &self.dir)?;
        if path.exists() {
            fs::remove_file(&path)
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "deduplicate")]
    /// Checks if the key is in the index.
    pub fn contains(&self, key: &Vec<u8>) -> bool {
        self.seen
            .lock()
            .unwrap_or_else(|_| {
                panic!(
                    "The bloom filter for '{key}' is poisoned; could not continue.",
                    key = self.key
                )
            })
            .contains(key)
    }

    /// Checks if the key is in the index, and if not, adds it.
    ///
    /// Returns `true` if the the set already contains the value;
    /// otherwise it is inserted, and `false` is returned.
    #[cfg(feature = "deduplicate")]
    pub fn contains_or_set(&self, key: Vec<u8>) -> bool {
        !self
            .seen
            .lock()
            .unwrap_or_else(|_| {
                panic!(
                    "The bloom filter for '{key}' is poisoned; could not continue.",
                    key = self.key
                )
            })
            .insert(key)
    }

    /// Adds a key to the index.
    ///
    /// This will only add the key if it is not already in the index; and performs the
    /// operation behind a Mutex.
    ///
    /// Returns `true` if the key was added, and `false` if it was already in the index.
    pub fn add(&self, item: Vec<u8>) -> io::Result<bool> {
        #[cfg(feature = "deduplicate")]
        if self.contains_or_set(item.clone()) {
            crate::debug!(
                "The key '{item}' is already in the index for '{prefix}', skipping.",
                item = String::from_utf8_lossy(&item),
                prefix = self.key,
            );
            return Ok(false);
        }

        crate::debug!(
            target: LOG_TARGET,
            "Adding the item '{item}' to the index for '{key}'.",
            item=String::from_utf8_lossy(&item),
            key=self.key,
        );

        let mut buffer = self.buffer.lock().unwrap_or_else(|_| {
            panic!(
                "The buffer for '{key}' is poisoned; could not continue.",
                key = self.key
            )
        });
        let _flushed = if buffer.len() + item.len() + 1 > MAX_SIZE {
            crate::debug!(
                target: LOG_TARGET,
                "The buffer for '{key}' is full at {size} bytes; flushing to file.",
                key=self.key,
                size=buffer.len(),
            );

            // The buffer is full; flush it to the file.
            self.flush_buffer(&mut buffer)?
        } else {
            0
        };

        // This key is new.
        buffer.extend_from_slice(&item);
        buffer.push(b'\n');

        Ok(true)
    }

    /// Flushes the provided buffer to the file, and returns the number of bytes written.
    ///
    /// Internal function; use [`IndexFile::flush`] instead.
    fn flush_buffer(&self, buffer: &mut Vec<u8>) -> io::Result<usize> {
        // Do not create a new file if the buffer is empty.
        if buffer.is_empty() {
            return Ok(0);
        }

        let mut file = self.open_for_write()?;

        let outgoing_buffer = mem::replace(buffer, Vec::with_capacity(MAX_SIZE));

        #[cfg(test)]
        assert_eq!(buffer.len(), 0);

        let written = file.write(&outgoing_buffer)?;
        crate::debug!(
            target: LOG_TARGET,
            "Flushed {written} bytes to {path:?}.",
            written=written,
            path=path_for_key(&self.key, &self.dir)?,
        );
        Ok(written)
    }

    /// Flushes the buffer to the file, and returns the number of bytes written.
    pub fn flush(&self) -> io::Result<usize> {
        let mut existing_buffer = self.buffer.lock().unwrap_or_else(|_| {
            panic!(
                "The buffer for '{key}' is poisoned; could not continue.",
                key = self.key
            )
        });

        self.flush_buffer(&mut existing_buffer)
    }

    /// Post-process the index.
    pub fn post_process(&mut self) -> io::Result<()> {
        crate::debug!(
            target: LOG_TARGET,
            "Post-processing the index for '{key}'.",
            key=self.key,
        );

        let flushed = self.flush()?;

        crate::debug!(
            target: LOG_TARGET,
            "Flushed {flushed} bytes for '{key}'.",
            key=self.key,
            flushed=flushed,
        );

        // TODO Add per-file deduplication here.
        #[cfg(not(feature = "deduplicate"))]
        {}

        Ok(())
    }
}

impl<const MAX_SIZE: usize> Drop for IndexFile<MAX_SIZE> {
    // Flush the buffer to the file before dropping.
    //
    // # Warning
    // The use of drop here means that `'static` [`PrefixIndex`] instances will not
    // flush their buffers at the end of the program. Any `'static` [`PrefixIndex`]
    // instances should be flushed manually.
    fn drop(&mut self) {
        crate::debug!(
            target: LOG_TARGET,
            "Dropping the index for '{key}'; flushing the buffer.",
            key=self.key,
        );
        let _ = self.flush();
    }
}

#[cfg(test)]
#[cfg(not(feature = "skip_index_write"))]
mod test {
    use super::*;
    use std::io::BufRead;

    use rayon::prelude::*;

    use crate::config::TEST_DIR;

    #[test]
    fn sequential_write() {
        let key = "index_file_test";

        // Ensure drop is triggered.
        let path = '_index_scope: {
            let index = IndexFile::<256>::new(key.to_owned(), TEST_DIR).unwrap();
            index.dispose().expect("Could not dispose of index.");

            for i in 0..256 {
                let key = format!("test_{:03}", i).as_bytes().to_vec();
                index.add(key.clone()).unwrap();

                #[cfg(feature = "deduplicate")]
                assert!(index.contains(&key));
            }

            index.path().expect("Could not get path.")
        };

        let file = fs::OpenOptions::new()
            .read(true)
            .create(false)
            .open(&path)
            .expect("Could not open file.");

        let mut reader = io::BufReader::new(file);

        for i in 0..256 {
            let key = format!("test_{:03}", i);
            let mut line = String::new();
            crate::trace!(
                target: &(LOG_TARGET.to_owned() + "::sequential_write"),
                "Checking for key '{key}' in the index...",
            );
            reader.read_line(&mut line).expect("Could not read line.");
            assert_eq!(line.trim(), key);
        }

        fs::remove_file(&path).expect("Could not remove file.");
    }

    #[test]
    fn parallel_write() {
        let key = "index_file_test_parallel";

        let path = '_index_scope: {
            let index = IndexFile::<256>::new(key.to_owned(), TEST_DIR).unwrap();
            index.dispose().expect("Could not dispose of index.");

            let all_keys = (0..256)
                .map(|i| format!("test_{:03}", i).as_bytes().to_vec())
                .collect::<Vec<_>>();
            let chunks = all_keys.chunks(32);

            chunks.enumerate().par_bridge().for_each(|(_id, chunk)| {
                crate::debug!(
                    target: &(LOG_TARGET.to_owned() + "::parallel_write"),
                    "Processing chunk {id} with {size} keys.",
                    id=_id,
                    size=chunk.len(),
                );
                for key in chunk {
                    index.add(key.clone()).unwrap();

                    #[cfg(feature = "deduplicate")]
                    assert!(index.contains(key));
                }
            });

            index.path().expect("Could not get path.")
        };

        let file = fs::OpenOptions::new()
            .read(true)
            .create(false)
            .open(&path)
            .expect("Could not open file.");

        let reader = io::BufReader::new(file);
        let mut lines = Result::<Vec<_>, _>::from_iter(reader.lines())
            .expect("Could not read lines from file.");
        lines.sort();
        println!("{:?}", lines);

        lines.into_iter().enumerate().for_each(|(i, line)| {
            let key = format!("test_{:03}", i);
            assert_eq!(line.trim(), key);
        });

        fs::remove_file(&path).unwrap_or_else(|_| panic!("Could not remove file at '{path:?}'."));
    }
}
