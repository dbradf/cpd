use std::{collections::HashMap, fs, path::Path, time::Instant};

use ignore::{DirEntry, Walk};
use itertools::Itertools;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct Location {
    pub file: usize,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct CpdIndex {
    pub files: Vec<String>,
    pub lines: HashMap<String, Vec<Location>>,
}

impl CpdIndex {
    pub fn entry_with_dups(&self) -> HashMap<&String, &Vec<Location>> {
        self.lines.iter().filter(|&(_, v)| v.len() > 1).collect()
    }

    pub fn build_dup_map(&self) -> HashMap<usize, HashMap<usize, String>> {
        let mut file_map: HashMap<usize, HashMap<usize, String>> = HashMap::new();
        for (h, v) in self.entry_with_dups() {
            for l in v {
                if let Some(f) = file_map.get_mut(&l.file) {
                    f.insert(l.line, h.to_string());
                } else {
                    let mut line_hash = HashMap::new();
                    line_hash.insert(l.line, h.to_string());
                    file_map.insert(l.file, line_hash);
                }
            }
        }
        file_map
    }
}

pub fn build_n_gram_index(path: &Path, min_lines: usize) -> CpdIndex {
    let mut files = vec![];
    let mut lines: HashMap<String, Vec<Location>> = HashMap::new();
    let start = Instant::now();
    for result in Walk::new(path) {
        match result {
            Ok(entry) => hash_file(entry, &mut files, &mut lines, min_lines),
            Err(err) => eprintln!("ERROR: {}", err),
        }
    }

    let end = Instant::now();
    println!(
        "build map time: {}ms",
        end.duration_since(start).as_millis()
    );
    CpdIndex { files, lines }
}

pub fn hash_file(
    entry: DirEntry,
    files: &mut Vec<String>,
    lines: &mut HashMap<String, Vec<Location>>,
    min_lines: usize,
) {
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
        .collect::<Vec<_>>()
        .windows(min_lines)
        .for_each(|window| {
            let start_line = window.first().unwrap().0;
            let line = window.iter().map(|l| l.1.trim()).join("\n");
            if line.len() < 15 {
                return;
            }
            let h = hash_line(&line);
            if let Some(list) = lines.get_mut(&h) {
                list.push(Location {
                    file: file_index,
                    line: start_line + 1,
                })
            } else {
                lines.insert(
                    h,
                    vec![Location {
                        file: file_index,
                        line: start_line + 1,
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
