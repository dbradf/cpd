use std::{collections::HashMap, fs};

use ignore::{DirEntry, Walk};
use sha2::{Digest, Sha256};

#[derive(Debug)]
struct Location {
    file: usize,
    line: usize,
}

fn main() {
    let mut files = vec![];
    let mut lines: HashMap<String, Vec<Location>> = HashMap::new();
    for result in Walk::new(".") {
        match result {
            Ok(entry) => hash_file(entry, &mut files, &mut lines),
            Err(err) => eprintln!("ERROR: {}", err),
        }
    }

    for (_, v) in lines.iter().filter(|&(_, v)| v.len() > 1) {
        println!("Dup");
        for l in v {
            let filename = &files[l.file];
            println!("{}: line {}", filename, l.line);
        }
        println!();
    }
}

fn hash_file(entry: DirEntry, files: &mut Vec<String>, lines: &mut HashMap<String, Vec<Location>>) {
    if entry.file_type().unwrap().is_dir() {
        return;
    }
    files.push(entry.path().display().to_string());
    let file_index = files.len() - 1;
    let contents = fs::read_to_string(entry.path());
    if contents.is_ok() {
        match contents {
            Ok(t) => t,
            Err(_e) => return,
        }
        .lines()
        .enumerate()
        .for_each(|(i, l)| {
            let h = hash_line(l);
            if let Some(list) = lines.get_mut(&h) {
                list.push(Location {
                    file: file_index,
                    line: i + 1,
                })
            } else {
                lines.insert(
                    h,
                    vec![Location {
                        file: file_index,
                        line: i + 1,
                    }],
                );
            }
        });
    }
}

fn hash_line(line: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(line.trim());
    let result = hasher.finalize();

    hex::encode(result)
}
