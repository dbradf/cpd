use std::{
    cmp::{max, min},
    collections::HashMap,
    path::Path,
};

use crate::hash_file::{build_n_gram_index, Location};

mod hash_file;

fn main() {
    let cpd_index = build_n_gram_index(Path::new("."));
    let file_map = cpd_index.build_dup_map();

    cpd_index.files.iter().enumerate().for_each(|(i, f)| {
        let mut matches = matches_for_file(i, &file_map, &cpd_index.lines);
        if !matches.is_empty() {
            println!("\n{}:\n", f);

            matches.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
            for m in matches {
                let filename = &cpd_index.files[m.file];
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
