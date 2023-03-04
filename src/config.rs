use std::{fs, path::Path};

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;

const DEFAULT_MIN_LINES: usize = 4;

#[derive(Debug, Deserialize, Default)]
struct CpdConfigContents {
    pub min_lines: Option<usize>,
    pub ignored_globs: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct CpdConfig {
    contents: CpdConfigContents,
    ignore_matcher: Option<GlobSet>,
}

impl CpdConfig {
    fn new(contents: CpdConfigContents) -> Self {
        let ignore_matcher = contents
            .ignored_globs
            .as_ref()
            .map(|globs| build_ignore_matcher(globs));

        Self {
            contents,
            ignore_matcher,
        }
    }

    pub fn from_json_file(file_name: &Path) -> Self {
        let file_contents = fs::read_to_string(file_name).unwrap();
        let config: CpdConfigContents = serde_json::from_str(&file_contents).unwrap();

        Self::new(config)
    }

    pub fn default() -> Self {
        Self::new(CpdConfigContents::default())
    }

    pub fn get_min_lines(&self) -> usize {
        self.contents.min_lines.unwrap_or(DEFAULT_MIN_LINES)
    }

    pub fn should_file_be_ignored(&self, file: &Path) -> bool {
        if let Some(ignore_matcher) = &self.ignore_matcher {
            ignore_matcher.is_match(file)
        } else {
            false
        }
    }
}

fn build_ignore_matcher(ignored_globs: &[String]) -> GlobSet {
    let mut glob_builder = GlobSetBuilder::new();
    ignored_globs.iter().for_each(|g| {
        glob_builder.add(Glob::new(g).unwrap());
    });
    glob_builder.build().unwrap()
}
