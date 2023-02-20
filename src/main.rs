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

    let dup_map = lines.iter().filter(|&(_, v)| v.len() > 1);
    let mut file_map: HashMap<String, HashMap<usize, String>> = HashMap::new();

    for (h, v) in dup_map {
        for l in v {
            let filename = &files[l.file];
            if let Some(f) = file_map.get_mut(filename) {
                f.insert(l.line, h.to_string());
            } else {
                let mut line_hash = HashMap::new();
                line_hash.insert(l.line, h.to_string());
                file_map.insert(filename.to_string(), line_hash);
            }
        }
    }

    for f in &files {
        if let Some(locations) = file_map.get(f) {
            let mut file_printed = false;
            for (line, hash) in locations {
                if locations.get(&(line + 1)).is_some() && locations.get(&(line + 2)).is_some() {
                    let mut i = line + 2;
                    while locations.get(&i).is_some() {
                        i += 1;
                    }
                    if !file_printed {
                        println!("{}:\n", f);
                        file_printed = true;
                    }
                    println!("  {}-{}: {}", line, i, hash);
                    // if let Some(matches) = lines.get(hash) {
                    //     for x in matches {
                    //         let filename = &files[x.file];
                    //         println!(" - {}: {}", filename, x.line);
                    //     }
                    // }
                }
            }
        }
    }

    // for (_, v) in lines.iter().filter(|&(_, v)| v.len() > 1) {
    //     println!("Dup");
    //     for l in v {
    //         let filename = &files[l.file];
    //         println!("{}: line {}", filename, l.line);
    //     }
    //     println!();
    // }
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
