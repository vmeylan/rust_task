use ethers::{
    core::types::{Filter},
    providers::{Provider, Ws},
    prelude::*,
    abi::{Abi, EventExt, Detokenize},
    utils::keccak256,
};
use eyre::Result;
use dotenv::dotenv;
use serde::{Serialize, Deserialize};
use chrono::{Utc, NaiveDate, Datelike};
use std::io::Write;
use std::collections::HashMap;

mod etherscan;
mod test_sig_match;
mod data_store;
mod log_processing;
mod utils;
use crate::data_store::store_decoded_data;
use crate::log_processing::process_log;
use crate::utils::pretty_print_log;


// resources:
// https://doc.rust-lang.org/book/
// https://www.gakonst.com/ethers-rs/subscriptions/logs.html?highlight=abi#subscribing-to-logs
// https://docs.infura.io/networks/ethereum/json-rpc-methods/eth_getlogs
// https://www.gakonst.com/ethers-rs/subscriptions/multiple-subscriptions.html


/// fetch_eth_logs Fetches Ethereum logs for a given contract address and processes each log.
///
/// The function connects to the Ethereum network using a provider and creates
/// a filter to fetch logs for the given contract address. Each log is then processed
/// using the provided ABI.
///
/// # Arguments
///
/// * `address` - The Ethereum contract address for which logs are to be fetched.
/// * `abi` - The ABI containing event definitions to decode the logs.
///
/// # Returns
///
/// A Result indicating the success or failure of the fetching and processing.
async fn fetch_eth_logs(address: &str, abi: &Abi) -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key: String = std::env::var("INFURA_API_KEY").expect("INFURA_API_KEY not set");
    let url: String = format!("wss://mainnet.infura.io/ws/v3/{}", api_key);

    let provider = Provider::<Ws>::connect(url).await?;

    // Specify the filter
    let filter = Filter {
        address: Some(vec![address.parse()?].into()),
        ..Default::default()
    };

    // Get the logs specifically for the given address
    let mut logs_stream = provider.watch(&filter).await?;

    // Flag to track if the event_map has been created
    let mut map_created = false;
    // Create an empty HashMap to store the Keccak256 hash of event signatures as the key,
    // and a tuple of event name and the event structure as the value.
    let mut event_map = HashMap::new();

    loop { // Changed to an infinite loop
        if let Some(log) = logs_stream.next().await {
            // println!("Mock Log:\n{:?}", pretty_print_log(&log));  // used for unit test creation
            if !map_created {
                // /!\ We use event.abi_signature() instead of event.signature() here.
                // The reason is that `event.signature()` provides a human-readable format,
                // while `event.abi_signature()` provides the human-readable ABI signature
                // format suitable for hashing to match Ethereum's log signature standard.
                // https://docs.rs/ethers/latest/ethers/abi/struct.Event.html
                for (event_name, events) in &abi.events {
                    for event in events {
                        let event_signature_hash = keccak256(event.abi_signature().as_bytes());
                        event_map.insert(event_signature_hash, (event_name.clone(), event.clone()));
                    }
                }
                map_created = true;
            }

            let decoded_data = process_log(log, &event_map).await?;
            if let Some(data) = decoded_data {
                if let Err(e) = store_decoded_data(address, &data) {
                    eprintln!("Error storing decoded data: {}", e);
                }
            }
        }
    }
    Ok(())
}


#[tokio::main]
async fn main() {
    // Get the root directory
    let root_directory = match utils::root_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("Error: Root directory not found");
            return;
        }
    };

    // Construct the full path to abi.json using the root directory
    let abi_path = format!("{}/src/abi.json", root_directory);

    // Check if the file exists
    if let Ok(abi_json) = std::fs::read_to_string(abi_path) {
        let abi: ethers::abi::Abi = serde_json::from_str(&abi_json).unwrap();

        // Continue with fetching Ethereum logs using the ABI
        let address = "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640";
        if let Err(err) = fetch_eth_logs(address, &abi).await {
            eprintln!("Error: {}", err);
        }
    } else {
        eprintln!("Error: Failed to read ABI JSON file");
    }
}

