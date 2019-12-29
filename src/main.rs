#![no_main]

const USAGE: &str = "Remove duplicate lines from text file

USAGE: <path>...
";

const EXT: &str = "dedup.temp";
#[cfg(not(windows))]
const NEW_LINE: &[u8] = b"\n";
#[cfg(windows)]
const NEW_LINE: &[u8] = b"\r\n";

use std::fs;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::io::{BufReader, BufRead, BufWriter, Write};

fn caclulate_hash(line: &str) -> u64 {
    use std::hash::{Hasher};

    let mut hasher = DefaultHasher::new();
    hasher.write(line.as_bytes());
    hasher.finish()
}

#[no_mangle]
unsafe extern "C" fn main(argc: isize, argv: *const *const u8) -> isize {
    if argc <= 1 {
        println!("{}", USAGE);
        return 1;
    }

    let args = c_ffi::Args::new(argc, argv).expect("To get function arguments");

    'args: for arg in args.into_iter().skip(1) {
        let file = match fs::File::open(arg) {
            Ok(file) => file,
            Err(error) => {
                eprintln!("'{}': {}", arg, error);
                continue;
            }
        };

        let new_file_path = format!("{}.{}", arg, EXT);
        let mut new_file = match fs::File::create(&new_file_path) {
            Ok(file) => BufWriter::new(file),
            Err(error) => {
                eprintln!("'{}': Cannot create file. Error: {}", new_file_path, error);
                continue;
            }
        };

        let mut count = 0;
        let mut store = HashSet::new();

        for line in BufReader::new(file).lines() {
            let line = match line {
                Ok(line) => line,
                Err(error) => {
                    eprintln!("'{}': Unable to read. Error: {}", arg, error);
                    continue 'args;
                }
            };

            match store.insert(caclulate_hash(&line)) {
                true => match new_file.write(line.as_bytes()).and_then(|_| new_file.write(NEW_LINE)) {
                    Ok(_) => (),
                    Err(error) => {
                        eprintln!("'{}': Unable to write new file: Error: {}", new_file_path, error);
                        continue 'args;
                    }
                },
                false => {
                    count += 1;
                    continue;
                }
            }
        }

        drop(new_file);

        match match count {
            0 => fs::remove_file(new_file_path),
            _ => fs::rename(new_file_path, arg),
        } {
            Ok(_) => (),
            Err(error) => {
                eprintln!("'{}': Unable to write new file. Error: {}", arg, error);
            }
        }
    }

    0
}
