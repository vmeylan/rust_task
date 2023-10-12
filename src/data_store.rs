use std::io::{self, Write};
use std::path::Path;
use serde_json;
use chrono::{Utc, NaiveDateTime, Datelike};
use serde::{Serialize, Deserialize};
use crate::utils;


#[derive(Debug, Serialize, Deserialize)]
pub struct DecodedData {
    pub transaction_hash: String,
    pub sender: String,
    pub recipient: String,
    pub amount0: i128,
    pub amount1: i128,
    pub sqrtPriceX96: u128,
    pub liquidity: u128,
    pub tick: i32,
}


pub fn store_decoded_data(address: &str, data: &DecodedData) -> Result<(), io::Error> {
    let root_directory = match utils::root_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("Error: Root directory not found");
            return Err(io::Error::new(io::ErrorKind::Other, "Root directory not found"));
        }
    };

    // Construct the full path to the data directory using the root directory
    let data_dir = format!("{}/data", root_directory);

    // Check if the directory exists, and create it if it doesn't
    if !std::path::Path::new(&data_dir).exists() {
        if let Err(err) = std::fs::create_dir_all(&data_dir) {
            eprintln!("Error: Failed to create data directory: {}", err);
            return Err(err);
        }
    }

    // Get the current date and format it as yyyy_mm_dd
    let now = Utc::now().naive_utc();
    let formatted_date = format!("{}_{}_{}", now.year(), now.month(), now.day());

    // Create the filename using the address and date
    let filename = format!("{}/{}_{}_decoded_swaps.json", data_dir, address, formatted_date);

    // Serialize the data to JSON
    let json = serde_json::to_string(&data)?;

    // Check if the file exists. If it does, append a newline before the new JSON entry.
    // If not, just write the JSON entry to the new file.
    if Path::new(&filename).exists() {
        let mut file = std::fs::OpenOptions::new().append(true).open(filename)?;
        writeln!(file, "\n{}", json)?;
    } else {
        std::fs::write(&filename, json)?;
    }

    Ok(())
}
