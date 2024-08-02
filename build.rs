use std::{io, path};

static AUTOMATON_TEMPLATE: &str = include_str!("src/parse/_templates/automaton_template.rs");

/// Generate a automaton template.
fn generate_automaton_template(source: path::PathBuf, output: path::PathBuf) -> io::Result<()> {
    let name = source.file_stem().unwrap().to_string_lossy().to_string();
    let string = std::fs::read_to_string(&source)?;
    let words = string.split(char::is_whitespace);

    let template = AUTOMATON_TEMPLATE
        .replace("\"{{WORD_LIST}}\"", {
            &words.fold(String::new(), |text, word| {
                text + "\"" + word + "\",\n        "
            })
        })
        .replace("{{LIST_NAME}}", &name);

    std::fs::write(output, template)?;

    Ok(())
}

// Example custom build script.
fn main() -> io::Result<()> {
    // Tell Cargo that if the english words changes, to rerun this build script.
    println!("cargo::rerun-if-changed=data/words/en_common_words.csv");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=src/parse/_templates/automaton_template.rs");

    let out_dir = std::env::var("OUT_DIR").expect(
        "Failed to get the output directory. Please make sure that the environment variable `OUT_DIR` is set.",
    );

    let source = path::PathBuf::from("data/words/en_common_words.csv");
    let output = path::PathBuf::from(out_dir).join("en_common_words.rs");

    generate_automaton_template(source, output)?;

    Ok(())
}
