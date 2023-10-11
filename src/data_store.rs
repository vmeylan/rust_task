use std::fs;
use std::io::{self, Write};
use std::path::Path;
use serde_json;
use chrono::{Utc, NaiveDateTime, Datelike};
use serde::{Serialize, Deserialize};


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
    // Create the src/data directory if it doesn't exist
    let dir = Path::new("data/");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }

    // Get the current date and format it as yyyy_mm_dd
    let now = Utc::now().naive_utc();
    let formatted_date = format!("{}_{}_{}", now.year(), now.month(), now.day());

    // Create the filename using the address and date
    let filename = format!("{}/{}_{}_decoded_swaps.json", dir.display(), address, formatted_date);

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