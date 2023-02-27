use std::{
    cmp::{max, min},
    collections::HashMap,
    fs,
    time::Instant,
};

use ignore::{DirEntry, Walk};
use itertools::Itertools;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
struct Location {
    file: usize,
    line: usize,
}

fn main() {
    let mut files = vec![];
    let mut lines: HashMap<String, Vec<Location>> = HashMap::new();
    let start = Instant::now();
    for result in Walk::new(".") {
        match result {
            Ok(entry) => hash_file(entry, &mut files, &mut lines),
            Err(err) => eprintln!("ERROR: {}", err),
        }
    }

    let end = Instant::now();
    println!(
        "build map time: {}ms",
        end.duration_since(start).as_millis()
    );

    let dup_map = lines.iter().filter(|&(_, v)| v.len() > 1);
    let mut file_map: HashMap<usize, HashMap<usize, String>> = HashMap::new();

    for (h, v) in dup_map {
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

    files.iter().enumerate().for_each(|(i, f)| {
        let mut matches = matches_for_file(i, &file_map, &lines);
        if !matches.is_empty() {
            println!("\n{}:\n", f);

            matches.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
            for m in matches {
                let filename = &files[m.file];
                println!(
                    "- {} - {} :{}: {} - {}",
                    m.start, m.end, filename, m.remote_start, m.remote_end
                );
            }
        }
    });
}

#[derive(Debug)]
struct CopyPasteMatch {
    start: usize,
    end: usize,
    file: usize,
    remote_start: usize,
    remote_end: usize,
}

impl CopyPasteMatch {
    fn is_overlapping(&self, start: usize, end: usize, file: usize) -> bool {
        if file != self.file {
            return false;
        }
        if start >= self.start && start <= self.end {
            return true;
        }

        end >= self.start && end <= self.end
    }

    fn is_duplicate(&self, file: usize) -> bool {
        file == self.file && self.start == self.remote_start && self.end == self.remote_end
    }

    fn expand(&mut self, start: usize, end: usize, remote_start: usize, remote_end: usize) {
        self.start = min(self.start, start);
        self.end = max(self.end, end);
        self.remote_start = min(self.remote_start, remote_start);
        self.remote_end = max(self.remote_end, remote_end);
    }
}

fn matches_for_file(
    file_index: usize,
    file_map: &HashMap<usize, HashMap<usize, String>>,
    lines: &HashMap<String, Vec<Location>>,
) -> Vec<CopyPasteMatch> {
    let mut match_list: Vec<CopyPasteMatch> = vec![];
    if let Some(locations) = file_map.get(&file_index) {
        for (line, hash) in locations {
            if let Some(matches) = lines.get(hash) {
                for x in matches {
                    let start = *line;
                    let end = start + 4;
                    let mut found = false;
                    for m in &mut match_list {
                        if m.is_overlapping(start, end, x.file) {
                            m.expand(start, end, x.line, x.line + 4);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        match_list.push(CopyPasteMatch {
                            start,
                            end,
                            file: x.file,
                            remote_start: x.line,
                            remote_end: x.line + 4,
                        })
                    }
                }
            }
        }
    }
    match_list
        .into_iter()
        .filter(|m| !m.is_duplicate(file_index))
        .collect()
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
        .tuple_windows::<(_, _, _, _)>()
        .for_each(|((i, l0), (_, l1), (_, l2), (_, l3))| {
            let line = [l0, l1, l2, l3].map(|s| s.trim()).join("\n");
            if line.len() < 15 {
                return;
            }
            let h = hash_line(&line);
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
