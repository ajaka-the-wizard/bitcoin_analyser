use anyhow::{Result, bail};
use bytemuck::{Pod, Zeroable};
use serde::Deserialize;

#[repr(C)]
#[derive(Debug, Deserialize, Pod, Zeroable, Copy, Clone)]
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
}

#[repr(C)]
#[derive(Debug, Deserialize, Pod, Zeroable, Copy, Clone)]
pub struct Analysed {
    pub timestamp: i64,
    pub close: f64,
    pub sma: f64,
}
pub fn update_analysis_result(
    output: &mut [Analysed],
    index: usize,
    update: impl FnOnce(&mut Analysed),
) -> Result<()> {
    let Some(record) = output.get_mut(index) else {
        bail!(
            "output index {index} is outside the assigned output region of length {}",
            output.len()
        );
    };

    update(record);
    Ok(())
}

pub const TOTAL_NUMBER_OF_THREADS: usize = 8;
pub const MOVING_AVERAGE: usize = 50;
pub const OUTPUT_FILE_NAME: &str = "output/output.bin";
pub const PROCESSED_FILE_NAME: &str = "output/conv.bin";
