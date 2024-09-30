const TEST_DIR: &str = "./.tests";
const TEST_FILE: &str = "test_document.txt";

use std::{fs, io};

/// Try getting the test document.
fn get_test_document() -> io::Result<fs::File> {
    let path = fs::canonicalize(TEST_DIR).unwrap_or_else(|_| {
        panic!(
            "Failed to canonicalize the test directory path at '{path}'; does it exist?",
            path = TEST_DIR
        )
    });
    let path = path.join(TEST_FILE);

    fs::File::open(&path).map_err(|err| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "Failed to open the test document at '{path}': {err}",
                path = path.display(),
                err = err
            ),
        )
    })
}

macro_rules! create_test {
    (
        $name:ident,
        $chunk_size:literal,
        $max_line:literal
        $(,)?
    ) => {
        #[test]
        fn $name() {
            let file = get_test_document().expect("Failed to get the test document.");

            const CHUNK_SIZE: usize = $chunk_size;
            assert!(
                $max_line <= CHUNK_SIZE,
                "The maximum line length is greater than the chunk size."
            );
            assert!(
                $max_line > 11,
                "The maximum line length is inappropriately small."
            );
            let mut reader = reader::FixedMemoryReader::<_, $max_line>::from_file(file, CHUNK_SIZE);

            let mut buffer = reader::utils::new_buffer(CHUNK_SIZE);

            let mut total_lines = 0;
            loop {
                match reader.take_until(b'\n', &mut buffer) {
                    Ok(0) => break,
                    Ok(len) => {
                        let chunk = unsafe { String::from_utf8_unchecked(buffer[..len].to_vec()) };
                        assert!(
                            chunk.len() <= CHUNK_SIZE,
                            "Line is longer than the chunk size: {:?}",
                            chunk
                        );
                        assert!(
                            chunk.ends_with('\n'),
                            "Line does not end with a newline character: {:?}",
                            chunk
                        );
                        chunk.split_whitespace().for_each(|line| {
                            assert_eq!(line, "0123456789");
                            total_lines += 1;
                        })
                    }
                    Err(err) => {
                        eprintln!("Failed to read from the test document: {}", err);
                        break;
                    }
                }
            }

            assert_eq!(
                total_lines, 200,
                "Total lines read does not match the expected value."
            );
        }
    };
}

create_test!(read_test_document_64_12, 64, 12);
create_test!(read_test_document_128_24, 128, 24);
create_test!(read_test_document_256_48, 256, 48);
create_test!(read_test_document_512_96, 512, 96);
create_test!(read_test_document_4096_12, 4096, 12);
