use std::sync::Arc;

use clap::Parser;
use rayon::prelude::*;
use reader::sync::MmapReader;
use rockyou2024::{config, models::IndexCollection};

#[cfg(feature = "progress")]
use kdam::{tqdm, BarExt};
#[cfg(feature = "progress")]
use std::sync::Mutex;

/// Index the input file.
fn index() -> anyhow::Result<()> {
    let args = rockyou2024::cli::CliArgs::parse();

    #[cfg(feature = "progress")]
    let file_size = std::fs::metadata(&args.input)
        .map_err(|err| {
            anyhow::Error::new(err).context(format!(
                "Failed to get metadata for input file: {}",
                args.input
            ))
        })?
        .len();

    #[cfg(feature = "progress")]
    let pbar = Arc::new(Mutex::new({
        let mut bar = tqdm!(
            total = file_size as usize,
            position = 0,
            desc = "Indexing",
            unit = "bytes",
            miniters = 1
        );

        bar.refresh().expect("Failed to refresh progress bar.");

        bar
    }));

    let reader = MmapReader::from_path(&args.input).map_err(|err| {
        anyhow::Error::new(err).context(format!("Failed to memory-map input file: {}", args.input))
    })?;

    let reader = if cfg!(feature = "progress") {
        reader.with_chunk_size(args.max_chunk_size)
    } else {
        reader.with_chunks(args.threads * 2)
    };

    let collection = Arc::new(IndexCollection::<
        { config::INDEX_LENGTH },
        { config::INDEX_DEPTH },
        // { config::MAX_INDEX_BUFFER_SIZE },
    >::new(args.output.into()));

    reader.iter::<b'\n'>().par_bridge().try_for_each(|chunk| {
        rockyou2024::info!(target: "ParBridgeProcessChunk", "Processing chunk of size: {}", chunk.len());
        let collection = Arc::clone(&collection);

        #[cfg(feature = "progress")]
        let pbar_local = Arc::clone(&pbar);

        let result = process_chunk(collection, chunk);

        #[cfg(feature = "progress")]
        pbar_local.lock().map_err(
            |_err| {
                anyhow::Error::msg("Failed to lock progress bar")
            }
        ).and_then(
            |mut pbar| {
                pbar.update(chunk.len()).and_then(
                    |_| pbar.refresh()
                ).map_err(
                    |err| {
                        anyhow::Error::new(err).context("Failed to update progress bar")
                    }
                )
            }
        ).unwrap_or_else(
            |err| {
                rockyou2024::error!("Failed to update progress bar: {}", err);
            }
        );

        result
    })
}

/// Process a chunk of data.
fn process_chunk(
    collection: Arc<
        IndexCollection<
            { config::INDEX_LENGTH },
            { config::INDEX_DEPTH },
            // { config::MAX_INDEX_BUFFER_SIZE },
        >,
    >,
    chunk: &[u8],
) -> anyhow::Result<()> {
    const LOG_TARGET: &str = "ProcessChunk";

    chunk
        .split(|&byte| byte == b'\n')
        .filter_map(|line| {
            // Remove lines that are too long; they would not be read correctly anyway.
            if line.len() > config::MAX_LINE_LENGTH {
                rockyou2024::warn!(
                    target: LOG_TARGET,
                    "Line too long ({} bytes); skipping.",
                    line.len()
                );
                return None;
            }

            Some(
                collection.add(line.to_vec()).map_err(|err| {
                    anyhow::Error::new(err).context("Failed to insert line into index")
                }),
            )
        })
        .for_each(
            // Do not panic on error; just log it.
            |result: Result<(), anyhow::Error>| {
                if let Err(error) = result {
                    rockyou2024::error!(target: LOG_TARGET, "{}", error);
                }
            },
        );

    Ok(())
}

fn main() {
    if let Err(err) = index() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    } else {
        rockyou2024::info!("Indexing completed successfully.");
    }
}
