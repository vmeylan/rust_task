mod etherscan;
mod test_sig_match;

use ethers::{
    core::types::{Filter, Log, H160, U256},
    providers::{Provider, Ws},
    prelude::*,
    abi::{Abi, RawLog, EventExt, Detokenize, Token},
    utils::keccak256,
};
use eyre::Result;
use dotenv::dotenv;
use serde::Deserialize;
use std::collections::HashMap;

// resources:
// https://www.gakonst.com/ethers-rs/subscriptions/logs.html?highlight=abi#subscribing-to-logs
// https://docs.infura.io/networks/ethereum/json-rpc-methods/eth_getlogs
// https://www.gakonst.com/ethers-rs/subscriptions/multiple-subscriptions.html helpful for the long run


fn print_decoded_params(decoded: &ethers::abi::Log, variable_names: &[&str]) {
    for param in &decoded.params {
        if variable_names.contains(&param.name.as_str()) {
            match param.name.as_str() {
                "amount0" | "amount1" => {
                    if let Token::Int(value) = &param.value {
                        if *value > U256::from(i128::MAX as u128) {
                            let neg_value = U256::max_value() - *value + U256::one();
                            println!("{}: -{}", param.name, neg_value);
                        } else {
                            println!("{}: {}", param.name, value.low_u128() as i128);
                        }
                    }
                },
                "sqrtPriceX96" | "liquidity" => {
                    if let Token::Uint(value) = &param.value {
                        println!("{}: {}", param.name, value.low_u128());
                    }
                },
                "tick" => {
                    if let Token::Int(value) = &param.value {
                        println!("{}: {}", param.name, value.low_u64() as i32);
                    }
                },
                _ => println!("Unknown parameter: {}", param.name)
            }
        }
    }
}


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

    // Create an empty HashMap to store the Keccak256 hash of event signatures as the key,
    // and a tuple of event name and the event structure as the value.
    let mut event_map = HashMap::new();

    // Iterate over each event in the ABI.
    for (event_name, events) in &abi.events {
        for event in events {
            // /!\ We use event.abi_signature() instead of event.signature() here.
            // The reason is that `event.signature()` provides a human-readable format,
            // while `event.abi_signature()` provides the human-readable ABI signature
            // format suitable for hashing to match Ethereum's log signature standard.
            // https://docs.rs/ethers/latest/ethers/abi/struct.Event.html
            let event_signature_hash = keccak256(event.abi_signature().as_bytes());
            if event_map.contains_key(&event_signature_hash) {
                println!("Duplicate hash detected for event: {}", event_name);
            }
            event_map.insert(event_signature_hash, (event_name.clone(), event));
        }
    }

    let mut successfully_decoded = false;

    // Iterate over each event signature hash in our map.
    for (hash, (event_name, event)) in &event_map {
        // check if the event_name is equal to Swap
        if event_name != "Swap" {
            println!("Skipping event: {}", event_name);
            continue;
        }
        // Check if the first topic of the log (which is the event signature) matches the current hash.
        if log_topic.as_bytes() == *hash {
            // If the log's topic matches an event's signature, attempt to parse the raw log using the event's ABI details.
            // If the parsing fails, it might be due to reasons like a mismatched or outdated ABI, corrupted log data,
            // non-standard encoding, or other discrepancies between the log and the ABI definition.
            let result = event.parse_log(raw_log.clone()).map_err(|e| eyre::eyre!("Failed to decode event: {:?}", e));

            if let Ok(decoded) = result {
                println!("\nSuccessfully decoded event: {event_name}");

                // TODO write to CSV
                // TODO unit tests
                let variable_names = ["amount0", "amount1", "sqrtPriceX96", "liquidity", "tick"];
                print_decoded_params(&decoded, &variable_names);

                successfully_decoded = true;
                println!("{}", '\n');
                break;  // We break here since we successfully decoded the event
            } else {
                println!("Failed to decode event with signature: {}", event.signature());
            }
        }

        if !successfully_decoded {
            // println!("Failed to decode any event for this log.");
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

