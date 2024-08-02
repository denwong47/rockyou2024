use std::sync::{Arc, Mutex};

use clap::Parser;
use kdam::{tqdm, BarExt};
use rayon::prelude::*;
use reader::sync::MmapReader;
use rockyou2024::{config, models::IndexCollection};

/// Index the input file.
fn index() -> anyhow::Result<()> {
    let args = rockyou2024::cli::CliArgs::parse();

    let file_size = std::fs::metadata(&args.input)
        .map_err(|err| {
            anyhow::Error::new(err).context(format!(
                "Failed to get metadata for input file: {}",
                args.input
            ))
        })?
        .len();

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

    let reader = MmapReader::from_path(&args.input)
        .map_err(|err| {
            anyhow::Error::new(err)
                .context(format!("Failed to memory-map input file: {}", args.input))
        })?
        .with_chunk_size(args.max_chunk_size);

    let collection = Arc::new(IndexCollection::<
        { config::INDEX_LENGTH },
        { config::INDEX_DEPTH },
        { config::MAX_INDEX_BUFFER_SIZE },
    >::new(args.output.into()));

    #[allow(unused_variables)]
    reader.iter::<b'\n'>().par_bridge().try_for_each(|chunk| {
        rockyou2024::info!(target: "ParBridgeProcessChunk", "Processing chunk of size: {}", chunk.len());
        let collection = Arc::clone(&collection);
        let pbar_local = Arc::clone(&pbar);

        let result = process_chunk(collection, chunk);

        pbar_local.lock().map_err(
            |err| {
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
            { config::MAX_INDEX_BUFFER_SIZE },
        >,
    >,
    chunk: &[u8],
) -> anyhow::Result<()> {
    const LOG_TARGET: &str = "ProcessChunk";

    chunk
        .split(|&byte| byte == b'\n')
        .map(|line| {
            let line = std::str::from_utf8(line).map_err(|err| {
                anyhow::Error::new(err).context(format!(
                    "Failed to convert line to UTF-8: {text}",
                    text = String::from_utf8_lossy(line)
                ))
            })?;

            collection.add(line).map_err(|err| {
                anyhow::Error::new(err).context("Failed to insert line into index")
            })?;

            Ok(())
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
