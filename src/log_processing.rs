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
use hex::FromHex;


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


#[cfg(test)]
mod tests {
    use super::*;
    use ethers::core::types::{Log as EthersLog, H256};
    use ethers::abi::{ethabi, Token};
    use std::str::FromStr;

    #[test]
    fn test_process_log() {
        // 1. Set up a log to be processed. From log printed with pretty_print_log trait
        let log = Log {
            address: H160::from_str("0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640").unwrap(),
            topics: vec![
                H256::from_str("0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67").unwrap(),
                H256::from_str("0x000000000000000000000000d7f3fbe8c72a961a5515203eada59750437fa762").unwrap(),
                H256::from_str("0x0000000000000000000000001c09a10047fcc944efde9226e259eddfde2c1cf0").unwrap()
            ],
            data: Bytes::from_hex("0x0000000000000000000000000000000000000000000000000000000d92cae287fffffffffffffffffffffffffffffffffffffffffffffffdfe6d04e32064349f0000000000000000000000000000000000006270c87ad64fc69a7baa1492b4f20000000000000000000000000000000000000000000000017c7599806e23275900000000000000000000000000000000000000000000000000000000000317ce").unwrap(),
            block_hash: Some(H256::from_str("0x1a65b8bb49fe739ae92ed688ab765cafe4dbcdd2b6c442e48a682ce2c0e451ee").unwrap()),
            block_number: Some(U64::from(18326572)),
            transaction_hash: Some(H256::from_str("0x13f84c56285e67f705bca6cb865610deda492752c0face651e0b3cb7893500f3").unwrap()),
            transaction_index: Some(U64::from(7)),
            log_index: Some(U256::from(49)),
            transaction_log_index: None,
            log_type: None,
            removed: Some(false),
        };

        // 2. Set up the event map
        let wrapped_json = std::fs::read_to_string("src/abi.json").unwrap();
        let abi: ethers::abi::Abi = serde_json::from_str(&wrapped_json).unwrap();
        let mut event_map = HashMap::new();
        for (event_name, events) in &abi.events {
            for event in events {
                let event_signature_hash = keccak256(event.abi_signature().as_bytes());
                event_map.insert(event_signature_hash, (event_name.clone(), event.clone()));
            }
        }

        // 3. Call the process_log function
        let result = tokio_test::block_on(process_log(log, &event_map));

        // 4. Check the result
        assert!(result.is_ok());
        let decoded_data = result.unwrap();
        assert!(decoded_data.is_some());

        let data = decoded_data.unwrap();
        // https://etherscan.io/tx/0x13f84c56285e67f705bca6cb865610deda492752c0face651e0b3cb7893500f3#eventlog
        assert_eq!(data.transaction_hash, "0x13f84c56285e67f705bca6cb865610deda492752c0face651e0b3cb7893500f3");
        assert_eq!(data.sender, "0xd7f3fbe8c72a961a5515203eada59750437fa762");
        assert_eq!(data.recipient, "0x1c09a10047fcc944efde9226e259eddfde2c1cf0");
        assert_eq!(data.amount0, 58297344647);
        assert_eq!(data.amount1, -37006917189485972321);
        assert_eq!(data.sqrtPriceX96, 1996611740862433600358475292128498);
        assert_eq!(data.liquidity, 27414987083570423641);
        assert_eq!(data.tick, 202702);
    }
}