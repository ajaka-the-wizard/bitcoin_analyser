use std::{path::PathBuf, process};

use conv::conv_from_csv_to_bin;

pub mod conv;

pub fn start(args: Vec<String>) {
    let path = PathBuf::from(args[0].clone());
    if path.extension().unwrap() == "csv" {
        conv_from_csv_to_bin(path).unwrap();
        process::exit(0);
    };
}
