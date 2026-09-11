use anyhow::{Context, Result, bail};
use conv::conv_from_csv_to_bin;
use memmap2::{Mmap, MmapMut};
use std::fs::OpenOptions;
use std::{fs::File, path::PathBuf, thread};

use crate::analysis::sma::calculate_sma_chunk;
use crate::common::{Analysed, Candle, TOTAL_NUMBER_OF_THREADS};

pub mod analysis;
pub mod common;
pub mod conv;

pub fn start(args: Vec<String>) -> Result<()> {
    let [filename] = args.as_slice() else {
        anyhow::bail!("usage: bitcoin_analyser <input.csv>");
    };
    let path = PathBuf::from(filename);
    let processed_file = conv_from_csv_to_bin(path)?;
    start_analysis(processed_file).with_context(|| "Failed to begin file processing")?;
    Ok(())
}

pub fn start_analysis(file: File) -> Result<()> {
    let output_file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(true)
        .open(crate::common::OUTPUT_FILE_NAME)
        .with_context(|| "Could not open output file")?;

    let metadata = file.metadata().unwrap();
    if metadata.len() == 0 {
        bail!("Input file seems to be empty")
    }

    let mmaped_file = unsafe { Mmap::map(&file).with_context(|| "Could not memory map file")? };

    let candles: &[Candle] = bytemuck::cast_slice(&mmaped_file[..]);
    let length = candles.len();
    let output_file_size = length * std::mem::size_of::<Analysed>();
    output_file
        .set_len(output_file_size as u64)
        .with_context(|| "Could not set output file length")?;

    let mut output_mmap = unsafe {
        MmapMut::map_mut(&output_file).with_context(|| "Could not memory map output file")?
    };

    let output: &mut [Analysed] = bytemuck::cast_slice_mut(&mut output_mmap[..]);
    let worker_count = TOTAL_NUMBER_OF_THREADS;

    let chunk_size = length / worker_count;
    let remainder = length % worker_count;

    thread::scope(|scope| -> Result<()> {
        let mut remaining_output = output;
        let mut chunk_start = 0;
        let mut workers = Vec::with_capacity(worker_count);

        for worker_index in 0..worker_count {
            let current_chunk_size = chunk_size + usize::from(worker_index < remainder);
            let chunk_end = chunk_start + current_chunk_size;
            let (worker_output, rest) = remaining_output.split_at_mut(current_chunk_size);

            workers.push(scope.spawn(move || {
                // println!(
                //     "Thread {} owns candle indices [{}, {})",
                //     worker_index + 1,
                //     chunk_start,
                //     chunk_end
                // );
                analysis(candles, worker_output, chunk_start, chunk_end)
            }));

            remaining_output = rest;
            chunk_start = chunk_end;
        }

        for worker in workers {
            worker
                .join()
                .map_err(|_| anyhow::anyhow!("analysis worker panicked"))??;
        }

        Ok(())
    })?;

    output_mmap
        .flush()
        .with_context(|| "Could not flush output memory map")?;

    Ok(())
}

fn analysis(candles: &[Candle], output: &mut [Analysed], start: usize, end: usize) -> Result<()> {
    calculate_sma_chunk(candles, start, end, output, crate::common::MOVING_AVERAGE)?;
    Ok(())
}
