use rayon::prelude::*;
use std::{
    cmp::{max, min},
    collections::HashMap,
    fs,
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use serde::Serialize;

use crate::{
    config::CpdConfig,
    hash_file::{build_n_gram_index, Location},
};

mod config;
mod hash_file;

#[derive(Debug, Serialize)]
struct CpdMatch<'a> {
    start: usize,
    end: usize,
    matching_file: &'a str,
    match_start: usize,
    match_end: usize,
}

#[derive(Debug, Serialize)]
struct CpdReport<'a> {
    filename: &'a str,
    matches: Vec<CpdMatch<'a>>,
}

/// A command to detect copy/pasted code.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opts {
    /// File to write report to.
    #[arg(long)]
    report_file: Option<PathBuf>,

    /// Base directory to analyze from.
    #[arg(long, default_value = ".")]
    base_dir: PathBuf,

    /// Path to configuration file.
    #[arg(long)]
    config_file: Option<PathBuf>,
}

fn main() {
    let opts = Opts::parse();

    let config = if let Some(config_file) = &opts.config_file {
        CpdConfig::from_json_file(config_file)
    } else {
        CpdConfig::default()
    };

    let cpd_index = build_n_gram_index(&opts.base_dir, config.get_min_lines());
    let file_map = cpd_index.build_dup_map();

    let start = Instant::now();
    let report: Vec<CpdReport> = cpd_index
        .files
        .par_iter()
        .enumerate()
        .filter_map(|(i, f)| {
            let mut matches =
                matches_for_file(i, &file_map, &cpd_index.lines, config.get_min_lines());
            if !matches.is_empty() {
                matches.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
                Some(CpdReport {
                    filename: f,
                    matches: matches
                        .par_iter()
                        .map(|m| {
                            let filename = &cpd_index.files[m.file];
                            CpdMatch {
                                start: m.start,
                                end: m.end,
                                matching_file: filename,
                                match_start: m.remote_start,
                                match_end: m.remote_end,
                            }
                        })
                        .collect(),
                })
            } else {
                None
            }
        })
        .collect();
    let end = Instant::now();

    println!("report time: {}ms", end.duration_since(start).as_millis());
    let json_report = serde_json::to_string_pretty(&report).unwrap();
    if let Some(report_file) = opts.report_file {
        fs::write(report_file, json_report).expect("Error writing to file");
    } else {
        println!("{}", &json_report);
    }
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
    min_lines: usize,
) -> Vec<CopyPasteMatch> {
    let mut match_list: Vec<CopyPasteMatch> = vec![];
    if let Some(locations) = file_map.get(&file_index) {
        for (line, hash) in locations {
            if let Some(matches) = lines.get(hash) {
                for x in matches {
                    let start = *line;
                    let end = start + min_lines;
                    let mut found = false;
                    for m in &mut match_list {
                        if m.is_overlapping(start, end, x.file) {
                            m.expand(start, end, x.line, x.line + min_lines);
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
                            remote_end: x.line + min_lines,
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
