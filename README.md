A test to see if we can build something to search this new rockyou dump real quick.

## Getting the RockYou dump

See the [RockYou2024](https://github.com/exploit-development/RockYou2024) repository for
more information on how to get the dump.

The file is ~151GB in size uncompressed.

## Indexing

Since the ``rockyou`` dump is too large to fit into memory, we could not run Aho-Corasick on the entire dump; neither could a tree be built and read in a reasonable amount of time. Therefore indices must be built for the file prior to any searching.

By default, indices for a word are defined by:

- the word will be normalised as per UAX #15 rules, then all
    non-ascii-alphanumerics removed.
- all characters that are common substitutes in passwords will be
    replaced by their alphabet equivalents.
- if the word is shorter than 3 characters, no indices will be possible.
- the first index will be the first 3 characters of the word.
- for each of the most common 3,600 English words according to EF and
    Oxford Dictionary, each of their first 3 characters are taken. If
    those characters are identified in sequence in the word, they will be
    considered an index for the word.

The indices are stored in files with names in form of `subset_<index>.csv`. The original untransformed word is stored. Each of these files should be small enough for Aho-Corasick to be run on them.

To build the indices, run the following command:

```bash
cargo run --bin index --release
```
