//! Parse command line arguments.

use clap::Parser;

use crate::config;

/// Command line arguments.
#[derive(Parser, Debug, Clone)]
pub struct CliArgs {
    #[arg(short, long, default_value_t = config::SOURCE_PATH.to_owned())]
    pub input: String,

    #[arg(short, long, default_value_t = config::INDEX_PATH.to_owned())]
    pub output: String,

    #[arg(short, long, default_value_t = config::NUMBER_OF_THREADS)]
    pub threads: usize,

    #[arg(long, default_value_t = config::CHUNK_SIZE)]
    pub chunk_size: usize,

    #[arg(long, default_value_t = config::MAX_CHUNK_SIZE)]
    pub max_chunk_size: usize,
}
