mod etherscan;
mod test_sig_match;

use ethers::{
    core::types::{Filter, Log, H160, I256},
    providers::{Provider, Ws},
    prelude::*,
    abi::{Abi, RawLog, EventExt},
    utils::keccak256,
};
use eyre::Result;
use dotenv::dotenv;
use serde::Deserialize;
use std::collections::HashMap;


/// process_log Processes a given Ethereum log entry using the provided ABI.
///
/// This function attempts to decode the log entry based on known event signatures
/// from the ABI. If successful, it prints out the relevant event parameters.
///
/// # Arguments
///
/// * `log` - The Ethereum log entry to be processed.
/// * `abi` - The ABI containing event definitions to decode the log.
///
/// # Returns
///
/// A Result indicating the success or failure of the processing.
async fn process_log(log: Log, abi: &Abi) -> Result<(), Box<dyn std::error::Error>> {
    let raw_log = RawLog {
        topics: log.topics.clone(),
        data: (*log.data.clone()).to_vec(),
    };

    let log_topic: H256 = log.topics[0];
    let hex_string = "0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67";
    let parsed_hex: H256 = hex_string.parse().expect("Failed to parse hexadecimal string");

    let mut event_map = HashMap::new();
    for (event_name, events) in &abi.events {
        for event in events {
            // event.abi_signature() and not event.signature() instead of /!\
            // -> keccack256 and ethers::abi::EventExt::keccack256 from https://docs.rs/ethers/latest/ethers/abi/struct.Event.html
            let event_signature_hash = keccak256(event.abi_signature().as_bytes());
            if event_map.contains_key(&event_signature_hash) {
                println!("Duplicate hash detected for event: {}", event_name);
            }
            event_map.insert(event_signature_hash, (event_name.clone(), event));
        }
    }

    let mut successfully_decoded = false;
    for (hash, (event_name, event)) in &event_map {
        if log_topic.as_bytes() == *hash {
            let result = event.parse_log(raw_log.clone()).map_err(|e| eyre::eyre!("Failed to decode event: {:?}", e));

            if let Ok(decoded) = result {
                println!("Successfully decoded event: {event_name}");
                let amount0_token = decoded.params.iter().find(|param| param.name == "amount0");
                if let Some(token) = amount0_token {
                    let amount0: U256 = token.value.clone().into_int().unwrap();
                    println!("amount0: {}", amount0);
                } else {
                    println!("Could not find 'amount0' in decoded params.");
                }

                let amount1_token = decoded.params.iter().find(|param| param.name == "amount1");
                if let Some(token) = amount1_token {
                    let amount1: U256 = token.value.clone().into_int().unwrap();
                    println!("amount1: {}", amount1);
                } else {
                    println!("Could not find 'amount1' in decoded params.");
                }

                successfully_decoded = true;
                break;  // We break here since we successfully decoded the event
            } else {
                println!("Failed to decode event with signature: {}", event.signature());
            }
        }

        if !successfully_decoded {
            println!("Failed to decode any event for this log.");
        }
    }
    Ok(())
}



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

    while let Some(log) = logs_stream.next().await {
        if let Err(err) = process_log(log, &abi).await {
            eprintln!("Error processing log: {}", err);
        }
    }

    Ok(())
}


#[tokio::main]
async fn main() {
    // test_sig_match::test_hash();
    let address = "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640";

    let wrapped_json = std::fs::read_to_string("src/abi.json").unwrap();
    let abi: ethers::abi::Abi = serde_json::from_str(&wrapped_json).unwrap();

    // Fetch logs using the ABI
    if let Err(err) = fetch_eth_logs(address, &abi).await {
        eprintln!("Error: {}", err);
    }
}

