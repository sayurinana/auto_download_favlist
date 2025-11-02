use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;

use encoding_rs::{Encoding, UTF_8};
use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::errors::FavlistError;
use crate::models::VideoEntry;

pub type CsvRow = HashMap<String, String>;

pub const FIELDNAMES: [&str; 4] = ["timestamp", "bv_id", "title", "fav_name"];

fn resolve_encoding(label: &str) -> Result<&'static Encoding, FavlistError> {
    Encoding::for_label(label.as_bytes())
        .ok_or_else(|| FavlistError::Encoding(format!("不支持的编码: {label}")))
}

pub fn load_existing_bv_ids(path: &Path, encoding: &str) -> Result<HashSet<String>, FavlistError> {
    if !path.exists() {
        return Ok(HashSet::new());
    }
    let enc = resolve_encoding(encoding)?;
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(enc))
            .build(file),
    );
    let candidate_fields = ["bv_id", "BV号", "视频BV号"];
    let headers = reader.headers()?.clone();
    let bv_index = headers.iter().enumerate().find_map(|(idx, name)| {
        if candidate_fields.iter().any(|candidate| candidate == &name) || name == "bv_id" {
            Some(idx)
        } else {
            None
        }
    });

    let Some(index) = bv_index else {
        return Ok(HashSet::new());
    };

    let mut existing = HashSet::new();
    for record in reader.records() {
        let record = record?;
        if let Some(value) = record.get(index) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                existing.insert(trimmed.to_string());
            }
        }
    }
    Ok(existing)
}

pub fn read_csv_rows(path: &Path, encoding: &str) -> Result<Vec<CsvRow>, FavlistError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let enc = resolve_encoding(encoding)?;
    let file = File::open(path)?;
    let mut reader = csv::Reader::from_reader(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(enc))
            .build(file),
    );
    let headers = reader.headers()?.clone();
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record?;
        let mut row = HashMap::new();
        for (header, value) in headers.iter().zip(record.iter()) {
            row.insert(header.to_string(), value.trim().to_string());
        }
        rows.push(row);
    }
    Ok(rows)
}

pub fn write_entries(
    path: &Path,
    encoding: &str,
    entries: &[VideoEntry],
) -> Result<usize, FavlistError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let enc = resolve_encoding(encoding)?;
    let is_new_file = !path.exists();
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut writer = BufWriter::new(file);

    if is_new_file {
        write_record(&mut writer, enc, &FIELDNAMES)?;
    }

    for entry in entries {
        write_record(
            &mut writer,
            enc,
            &[
                entry.timestamp.as_str(),
                entry.bv_id.as_str(),
                entry.title.as_str(),
                entry.fav_title.as_str(),
            ],
        )?;
    }

    writer.flush()?;
    Ok(entries.len())
}

fn write_record<W: Write>(
    writer: &mut W,
    encoding: &'static Encoding,
    record: &[&str],
) -> Result<(), FavlistError> {
    let mut csv_writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(Vec::new());
    csv_writer.write_record(record)?;
    csv_writer.flush()?;
    let buffer = csv_writer
        .into_inner()
        .map_err(|err| FavlistError::Io(err.into_error()))?;
    if encoding == UTF_8 {
        writer.write_all(&buffer)?;
    } else {
        let utf8 =
            String::from_utf8(buffer).map_err(|err| FavlistError::Encoding(err.to_string()))?;
        let (encoded, _, had_errors) = encoding.encode(&utf8);
        if had_errors {
            return Err(FavlistError::Encoding("编码转换失败".into()));
        }
        writer.write_all(&encoded)?;
    }
    Ok(())
}
