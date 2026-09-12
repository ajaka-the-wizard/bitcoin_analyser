use anyhow::{Result, bail};

use crate::common::{Analysed, Candle, update_analysis_result};

fn calculate_initial_sum(data: &[Candle], end_index: usize, window: usize) -> f64 {
    let start = end_index + 1 - window;
    data[start..=end_index]
        .iter()
        .map(|candle| candle.close)
        .sum()
}

pub fn calculate_sma_chunk(
    data: &[Candle],
    chunk_start: usize,
    chunk_end: usize,
    output: &mut [Analysed],
    window: usize,
) -> Result<()> {
    if window == 0 {
        bail!("window must be greater than zero");
    }
    if chunk_start > chunk_end {
        bail!("chunk_start ({chunk_start}) cannot be greater than chunk_end ({chunk_end})");
    }
    if chunk_end > data.len() {
        bail!(
            "chunk_end ({chunk_end}) exceeds available candle data length ({})",
            data.len()
        );
    }
    if output.len() != chunk_end - chunk_start {
        bail!(
            "output length ({}) does not match chunk size ({}); range is [{}, {})",
            output.len(),
            chunk_end - chunk_start,
            chunk_start,
            chunk_end
        );
    }

    let mut rolling_total = 0.0;
    let mut initialized = false;

    for local_index in 0..output.len() {
        let global_index = chunk_start + local_index;
        let candle = &data[global_index];

        if global_index < window - 1 {
            update_analysis_result(output, local_index, |record| {
                record.timestamp = candle.timestamp;
                record.close = candle.close;
                record.sma = -1.0;
            })?;
            continue;
        }

        if !initialized {
            rolling_total = calculate_initial_sum(data, global_index, window);
            initialized = true;
        } else {
            let outgoing_close = data[global_index - window].close;
            rolling_total = rolling_total - outgoing_close + candle.close;
        }

        update_analysis_result(output, local_index, |record| {
            record.timestamp = candle.timestamp;
            record.close = candle.close;
            record.sma = rolling_total / window as f64;
        })?;
    }

    Ok(())
}
