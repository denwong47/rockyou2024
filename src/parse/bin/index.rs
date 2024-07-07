use clap::Parser;
use reader::sync::MmapReader;

fn index() -> anyhow::Result<()> {
    let args = rockyou2024::cli::CliArgs::parse();

    let reader = MmapReader::from_path(&args.input)
        .map_err(|err| {
            anyhow::Error::new(err)
                .context(format!("Failed to memory-map input file: {}", args.input))
        })?
        .with_chunks(args.threads);

    #[allow(unused_variables)]
    reader.iter::<b'\n'>().for_each(|chunk| todo!());

    Ok(())
}

fn main() {
    if let Err(err) = index() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    } else {
        println!("Indexing completed successfully.");
    }
}
