# Rockyou 2024 Password Dump Search Index

![Rust](https://github.com/denwong47/rockyou2024/actions/workflows/rust-CI.yml/badge.svg?branch=main)
![Go](https://github.com/denwong47/rockyou2024/actions/workflows/go-CI.yml/badge.svg?branch=main)

A personal project to allow for efficient searching of the massive `rockyou2024.txt` password dump using an index-based approach.

## Context

The project addresses the challenges of searching through the massive `rockyou2024.txt` password dump, which contains nearly 100 billion lines of allegedly cracked passwords designed for `hashcat` consumption. The uncompressed file is 151GB in size.

Given the sheer size of the file, conventional search methods are impractical. For example, using `grep` for binary files results in search times of at least 10 minutes per request, even without employing regular expressions or fuzzy matching. This inefficiency makes it difficult to perform effective searches within a reasonable timeframe.

A common algorithm for string searching is Aho-Corasick, which builds a tree-like data structure for all the words to be searched. However, in this case, the challenge lies in the fact that the haystack (the password dump) is known, not the needles (the search queries). Pre-compiling an automaton based on the entire dump is impractical for the following reasons:

1. **Size of the Automaton:** Splitting the password dump and pre-building the automaton with the passwords would result in a very large cached automaton, making it difficult to manage and utilize effectively.

2. **Search Flexibility:** Pre-built automatons typically allow for full string matches only. This limitation does not align with user expectations, as users often require more flexible search options, such as partial matches or pattern-based searches.

Our project aims to develop a more efficient and user-friendly solution for searching this extensive password dump.


## Getting the RockYou dump

[Internet Archives](https://archive.org/details/rockyou2024.zip) had an entry for the compressed file.

Alternatively see the [RockYou2024](https://github.com/exploit-development/RockYou2024) repository for
more information on how to get the dump.

## Indexing

To enable efficient searching of the password dump, the data must be indexed. Rather than creating a traditional index on the original file—which would require extensive buffer scanning and result in a massive index—we have opted to separate the dump into smaller segments, each mapped to a "key." This approach defines what we mean by an index file in this project. A password can appear in multiple index files if it is deemed relevant to multiple keys.

## Indexing

To enable efficient searching of the password dump, the data must be indexed. Rather than creating a traditional index on the original file—which would require extensive buffer scanning and result in a massive index—we have opted to separate the dump into smaller segments, each mapped to a "key." This approach defines what we mean by an index file in this project. A password can appear in multiple index files if it is deemed relevant to multiple keys.

### Indexing Process

By default, the index keys for a password are defined by the following process:

1. **Normalization:** The password is normalized according to the UAX #15 rules. All non-ASCII alphanumeric characters are removed in this step.

2. **Character Substitution:** Common character substitutions often found in passwords are replaced by their alphabetic equivalents. The substitution table is as follows:

   - `8` is replaced by `b`
   - `3` is replaced by `e`
   - `6` and `9` are replaced by `g`
   - `1` and `!` are replaced by `i`
   - `l` is replaced by `i`
   - `0` is replaced by `o`
   - `5` and `$` are replaced by `s`
   - `7` is replaced by `t`
   - `2` is replaced by `z`
   - `®` is replaced by `r`
   - `£`, `€` are replaced by `e`
   - `@` is replaced by `a`

3. **Minimum Length Requirement:** If the normalized password is shorter than three characters, no indices will be generated.

4. **Primary Index Key:** The primary index key is derived from the first three characters of the normalized password.

5. **Secondary Index Keys:** For each of the most common 3,600 English words, according to EF and the Oxford Dictionary, the first three characters are taken. If these characters appear in sequence within the password, they serve as additional index keys for that password.

This indexing strategy allows for a more manageable and efficient search process, enabling faster lookup times by narrowing down the search space through relevant segments.

### Index File Structure

The indices are stored in files named in the format `subset_<index>.csv`, where `<index>` corresponds to the specific key. These files contain the original, untransformed words and are designed to be small enough to efficiently run the Aho-Corasick algorithm on them.

### Building the Indices

To build the indices, run the following command:

```bash
cargo run --bin index --release
```

### Customizing Key Length
The default key length is set to three characters but can be modified before compilation in the configuration file. Details about configuring the index key length and other settings will be covered in the next section.


## Configuration

The configuration for the project is managed in three distinct areas: command-line arguments, compile-time configurations, and build features. Each area provides different levels of customization and control over the indexing and search processes.

### 1. Command-Line Arguments

The command-line interface (CLI) allows users to customize various parameters when running the application. The available options are:

- **`input`**: Specifies the path to the input file. By default, it uses a predefined source path specified in the configuration.

- **`output`**: Defines the path for the output index files. The default is a predefined index path in the configuration.

- **`threads`**: Sets the number of threads to use for processing. The default value is determined by the configuration, balancing performance and resource use.

- **`chunk_size`**: Determines the size of each data chunk processed. This affects the granularity of processing and memory usage, with a default value specified in the configuration.

- **`max_chunk_size`**: Specifies the maximum allowable size for each chunk of data. This is useful for controlling memory usage, with a default size defined in the configuration.

### 2. Compile-Time Configurations

These settings are defined at compile time and provide default values and constraints for the indexing process:

- **`INDEX_LENGTH`**: Specifies the default number of characters used as the index key. The default is set to 3 characters, but this can be adjusted in the configuration file.

- **`INDEX_DEPTH`**: Defines the depth of characters used for indexing each string, with a default depth of 1.

- **`MAX_INDEX_BUFFER_SIZE`**: Sets the maximum buffer size for each index file, which is `2^16` bytes by default. This determines how much data is stored before being processed or written to disk.

### 3. Build Features

The project supports build features that can modify the application's behavior:

- **`progress`**: This feature is enabled by default and provides a `tqdm` style progress bar that shows the number of bytes processed. While useful for tracking progress, it may introduce a slight slowdown due to its implementation with a Mutex. To disable this feature, use the `--no-default-features` flag when building the project.


## Search Features

The search functionality is __under development__, with plans to implement a robust and efficient search experience through a Go Huma API backend and a Next.js front end. The search process is to minimize latency, leveraging the previously described indexing strategy.

### Planned Search Functionality

- **Frontend Interaction**: As soon as the user types in the first three characters of their search query, these characters are converted into an index key. This key is used to identify which index file to load for searching.

- **Backend Processing**: The Go Huma API will handle requests from the frontend, loading the relevant index files based on the generated key.

- **Efficient Searching**: Once the appropriate index file is loaded, the [Aho-Corasick algorithm](https://github.com/cloudflare/ahocorasick) will be used to perform efficient pattern matching within the file. This algorithm is particularly suited for handling large datasets and multiple pattern searches.

- **Search Results**: The matched passwords will be returned as an array to the front end, where they can be displayed to the user in a user-friendly manner.


## Developer Notes

> [!TIP]
> If you encounter issues with `sequential_write` or `parallel_write` tests, but only during `pre-commit`, that is due to `pre-commit` being cancelled halfway and some cache files not being
purged correctly. Use `pre-commit clean` to start afresh, and the problem should be resolved.
