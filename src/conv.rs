use std::{
    fs::{File, OpenOptions, create_dir_all},
    io::Write,
    path::PathBuf,
};

use anyhow::{Context, Result};
use csv::ReaderBuilder;

use crate::common::{Candle, PROCESSED_FILE_NAME};

pub fn conv_from_csv_to_bin(filename: PathBuf) -> Result<File> {
    create_dir_all("output").with_context(|| "could not create output directory")?;
    let file = OpenOptions::new()
        .read(true)
        .create(false)
        .open(filename.clone())
        .with_context(|| format!("{} does not exists", filename.display()))?;
    let mut write_file = OpenOptions::new()
        .write(true)
        .read(true)
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
    Ok(write_file)
}
