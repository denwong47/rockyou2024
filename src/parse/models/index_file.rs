use std::{
    fs,
    io::{self, Write},
    mem, path,
    sync::Mutex,
};

use bloomfilter::Bloom;

const DEFAULT_MAX_BUFFER: usize = 16394;

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
    key: String,
    dir: path::PathBuf,
    bloom: Mutex<Bloom<String>>,
    buffer: Mutex<Vec<u8>>,
}

/// The prefix for index files.
const INDEX_PREFIX: &str = "subset_";

/// The suffix for index files.
const INDEX_EXTENSION: &str = "csv";

/// The target for the crate.
const LOG_TARGET: &str = "IndexFile";

/// Returns the path for the given key and path.
fn path_for_key(key: impl AsRef<str>, dir: impl AsRef<path::Path>) -> io::Result<path::PathBuf> {
    let mut path_buf = path::Path::new(dir.as_ref()).canonicalize()?;
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

impl<const MAX_SIZE: usize> IndexFile<MAX_SIZE> {
    /// Creates a new instance of [`IndexFile`].
    pub fn new(key: String, dir: impl AsRef<path::Path>) -> io::Result<Self> {
        crate::debug!(
            target: LOG_TARGET,
            "Creating a new index for '{key}' at {dir:?}.",
            key=key,
            dir=dir.as_ref(),
        );
        let bloom = Bloom::new(65536, 8192).into();
        let buffer = Vec::with_capacity(DEFAULT_MAX_BUFFER).into();

        fs::create_dir_all(&dir)?;
        Ok(Self {
            key,
            dir: dir.as_ref().to_owned(),
            bloom,
            buffer,
        })
    }

    /// Returns the path for the index file.
    pub fn path(&self) -> io::Result<path::PathBuf> {
        path_for_key(&self.key, &self.dir)
    }

    /// Open the file associated with the key.
    pub fn open(&self) -> io::Result<fs::File> {
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

    /// Checks if the key is in the index.
    pub fn contains(&self, key: &String) -> bool {
        self.bloom
            .lock()
            .unwrap_or_else(|_| {
                panic!(
                    "The bloom filter for '{key}' is poisoned; could not continue.",
                    key = self.key
                )
            })
            .check(key)
    }

    /// Checks if the key is in the index, and if not, adds it.
    pub fn contains_or_set(&self, key: &String) -> bool {
        self.bloom
            .lock()
            .unwrap_or_else(|_| {
                panic!(
                    "The bloom filter for '{key}' is poisoned; could not continue.",
                    key = self.key
                )
            })
            .check_and_set(key)
    }

    /// Adds a key to the index.
    ///
    /// This will only add the key if it is not already in the index; and performs the
    /// operation behind a Mutex.
    ///
    /// Returns `true` if the key was added, and `false` if it was already in the index.
    pub fn add(&self, item: String) -> io::Result<bool> {
        if !self.contains_or_set(&item) {
            crate::debug!(
                target: LOG_TARGET,
                "Adding the item '{item}' to the index for '{key}'.",
                item=item,
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
            buffer.extend_from_slice(item.as_bytes());
            buffer.push(b'\n');

            Ok(true)
        } else {
            crate::debug!(
                "The key '{item}' is already in the index for '{prefix}', skipping.",
                item = item,
                prefix = self.key,
            );
            Ok(false)
        }
    }

    /// Flushes the provided buffer to the file, and returns the number of bytes written.
    ///
    /// Internal function; use [`IndexFile::flush`] instead.
    fn flush_buffer(&self, buffer: &mut Vec<u8>) -> io::Result<usize> {
        let mut file = self.open()?;

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
mod test {
    use super::*;
    use std::io::BufRead;

    use rayon::prelude::*;

    const TEST_DIR: &str = "./.tests";

    #[test]
    fn sequential_write() {
        let key = "index_file_test";

        // Ensure drop is triggered.
        let path = '_index_scope: {
            let index = IndexFile::<256>::new(key.to_owned(), TEST_DIR).unwrap();
            index.dispose().expect("Could not dispose of index.");

            for i in 0..256 {
                let key = format!("test_{:03}", i);
                index.add(key.clone()).unwrap();
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
                .map(|i| format!("test_{:03}", i))
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

        fs::remove_file(&path).expect("Could not remove file.");
    }
}
