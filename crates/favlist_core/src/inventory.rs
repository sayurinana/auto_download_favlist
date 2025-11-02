use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::Regex;
use walkdir::WalkDir;

use crate::csv_utils::CsvRow;

lazy_static! {
    static ref BV_PATTERN: Regex = Regex::new(r"(BV[0-9A-Za-z]{10})").expect("BV正则");
}

pub fn extract_bvids(text: &str) -> HashSet<String> {
    BV_PATTERN
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

pub fn scan_directory_bvids(directory: &Path) -> io::Result<HashMap<String, Vec<PathBuf>>> {
    let mut mapping: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for entry in WalkDir::new(directory).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let path = entry.path().to_path_buf();
            let name = entry.file_name().to_string_lossy();
            for bvid in extract_bvids(&name) {
                mapping.entry(bvid).or_default().push(path.clone());
            }
        }
    }
    Ok(mapping)
}

pub fn write_inventory_file(
    directory: &Path,
    mapping: &HashMap<String, Vec<PathBuf>>,
) -> io::Result<PathBuf> {
    let inventory_path = directory.join("existing_videos.txt");
    let mut file = File::create(&inventory_path)?;
    writeln!(file, "# 文件名及对应BV号列表")?;
    if mapping.is_empty() {
        writeln!(file, "(未找到包含BV号的文件)")?;
    } else {
        let mut entries: Vec<_> = mapping.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        for (bvid, paths) in entries {
            for path in paths {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    writeln!(file, "{bvid}\t{name}")?;
                }
            }
        }
    }
    Ok(inventory_path)
}

fn extract_bvid_from_row(row: &CsvRow) -> Option<String> {
    for key in ["bv_id", "BV号", "视频BV号"] {
        if let Some(value) = row.get(key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

pub fn diff_new_entries(old_rows: &[CsvRow], new_rows: &[CsvRow]) -> Vec<CsvRow> {
    let old_set: HashSet<String> = old_rows.iter().filter_map(extract_bvid_from_row).collect();
    new_rows
        .iter()
        .filter(|row| {
            if let Some(bvid) = extract_bvid_from_row(row) {
                !old_set.contains(&bvid)
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

pub fn find_missing_videos(csv_rows: &[CsvRow], existing_bvids: &[String]) -> Vec<CsvRow> {
    let existing: HashSet<&str> = existing_bvids.iter().map(|s| s.as_str()).collect();
    csv_rows
        .iter()
        .filter(|row| {
            if let Some(bvid) = extract_bvid_from_row(row) {
                !existing.contains(bvid.as_str())
            } else {
                false
            }
        })
        .cloned()
        .collect()
}
