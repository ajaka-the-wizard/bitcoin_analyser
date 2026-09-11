use std::{fs::OpenOptions, io::Write, path::PathBuf};

use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use csv::ReaderBuilder;
use serde::Deserialize;
#[repr(C)]
#[derive(Debug, Deserialize, Pod, Zeroable, Copy, Clone)]
struct Candle {
    timestamp: i64,
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: f64,
}
const PROCESSED_FILE_NAME: &str = "output/conv.bin";

pub fn conv_from_csv_to_bin(filename: PathBuf) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .create(false)
        .open(filename.clone())
        .with_context(|| format!("{} does not exists", filename.display()))?;
    let mut write_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(PROCESSED_FILE_NAME)
        .with_context(|| format!("could not create file {}", PROCESSED_FILE_NAME))?;
    let mut reader = ReaderBuilder::new().from_reader(file);
    let mut lines = 0;
    for line in reader.deserialize() {
        let record: Candle = line?;
        write_file.write_all(bytemuck::bytes_of(&record))?;
        lines += 1;
    }
    println!(
        "Suceesfully converted and written {} lines of {} to {}",
        lines,
        filename.file_name().unwrap().display(),
        PROCESSED_FILE_NAME
    );
    Ok(())
}
