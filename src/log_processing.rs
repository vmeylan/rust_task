use ethers::{
    core::types::{Filter, Log, H160, U256},
    providers::{Provider, Ws},
    prelude::*,
    abi::{Abi, RawLog, EventExt, Detokenize, Token, ethabi, Event},
    utils::keccak256,
};
use ethers::types::Log as EthersLog;
use eyre::Result;
use dotenv::dotenv;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::io;
use std::path::Path;
use chrono::{Utc, NaiveDate, Datelike};
use std::io::Write;

use crate::data_store::DecodedData;
use crate::data_store::store_decoded_data;


// Convert a slice of u8 into a hexadecimal string representation.
pub fn to_hex(slice: &[u8]) -> String {
    format!("0x{}", hex::encode(slice))
}

pub fn parse_decoded_log(decoded: ethabi::Log, log: &EthersLog) -> Option<DecodedData> {
    // Extract the last 20 bytes of the topic, representing the Ethereum address,
    // because Ethereum addresses are 20 bytes long and topics are zero-padded.
    // Convert topics to Ethereum addresses.
    let sender = to_hex(&log.topics[1][12..]);
    let recipient = to_hex(&log.topics[2][12..]);

    // Convert transaction hash to its full hexadecimal string representation.
    let transaction_hash = to_hex(&log.transaction_hash.unwrap().0);

    let mut amount0: i128 = 0;
    let mut amount1: i128 = 0;
    let mut sqrtPriceX96: u128 = 0;
    let mut liquidity: u128 = 0;
    let mut tick: i32 = 0;

    for param in &decoded.params {
        match param.name.as_str() {
            "amount0" | "amount1" => {
                if let Token::Int(value) = &param.value {
                    let converted_value = if *value > U256::from(i128::MAX as u128) {
                        let neg_value = (U256::max_value() - *value + U256::one()).low_u128();
                        -(neg_value as i128)
                    } else {
                        value.low_u128() as i128
                    };

                    if param.name.as_str() == "amount0" {
                        amount0 = converted_value;
                    } else {
                        amount1 = converted_value;
                    }
                }
            }
            "sqrtPriceX96" => {
                if let Token::Uint(value) = &param.value {
                    sqrtPriceX96 = value.low_u128();
                }
            }
            "liquidity" => {
                if let Token::Uint(value) = &param.value {
                    liquidity = value.low_u128();
                }
            }
            "tick" => {
                if let Token::Int(value) = &param.value {
                    tick = value.low_u64() as i32;
                }
            }
            _ => {}
        }
    }

    Some(DecodedData {
        transaction_hash,
        sender,
        recipient,
        amount0,
        amount1,
        sqrtPriceX96,
        liquidity,
        tick,
    })
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
pub async fn process_log(log: Log, event_map: &HashMap<[u8; 32], (String, Event)>) -> Result<Option<DecodedData>, Box<dyn std::error::Error>> {
    let raw_log = RawLog {
        topics: log.topics.clone(),
        data: (*log.data.clone()).to_vec(),
    };

    let log_topic: H256 = log.topics[0];

    let mut successfully_decoded = false;

    // Iterate over each event signature hash in our map.
    for (hash, (event_name, event)) in event_map {
        // check if the event_name is equal to Swap
        if event_name != "Swap" {
            // println!("Skipping event: {}", event_name);
            continue;
        }
        // Check if the first topic of the log (which is the event signature) matches the current hash.
        if log_topic.as_bytes() == *hash {
            // If the log's topic matches an event's signature, attempt to parse the raw log using the event's ABI details.
            // If the parsing fails, it might be due to reasons like a mismatched or outdated ABI, corrupted log data,
            // non-standard encoding, or other discrepancies between the log and the ABI definition.
            let result = event.parse_log(raw_log.clone()).map_err(|e| eyre::eyre!("Failed to decode event: {:?}", e));

            let mut decoded_data = None;

            if let Ok(decoded) = result {
                decoded_data = parse_decoded_log(decoded, &log);
                if let Some(ref data) = decoded_data {
                    println!("{:?}", data);
                }
            }
            return Ok(decoded_data);
        }
    }
    Ok(None)
}