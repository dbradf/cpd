use std::{fs, path::Path};

use serde::Deserialize;

const DEFAULT_MIN_LINES: usize = 4;

#[derive(Debug, Deserialize, Default)]
pub struct CpdConfig {
    min_lines: Option<usize>,
}

impl CpdConfig {
    pub fn from_json_file(file_name: &Path) -> Self {
        let file_contents = fs::read_to_string(file_name).unwrap();

        serde_json::from_str(&file_contents).unwrap()
    }

    pub fn get_min_lines(&self) -> usize {
        self.min_lines.unwrap_or(DEFAULT_MIN_LINES)
    }
}
