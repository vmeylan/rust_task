use ethers::{
    core::types::{Filter},
    providers::{Provider, Ws},
    prelude::*,
    abi::{Abi, EventExt, Detokenize},
};
use eyre::Result;
use dotenv::dotenv;
use serde::{Serialize, Deserialize};
use chrono::{Utc, NaiveDate, Datelike};
use std::io::Write;

mod etherscan;
mod test_sig_match;
mod data_store;
mod log_processing;
use crate::data_store::store_decoded_data;
use crate::log_processing::process_log;


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

    while let Some(log) = logs_stream.next().await {
        let decoded_data = process_log(log, &abi).await?;
        if let Some(data) = decoded_data {
            if let Err(e) = store_decoded_data(address, &data) {
                eprintln!("Error storing decoded data: {}", e);
            }
        }
    }

    Ok(())
}


#[tokio::main]
async fn main() {
    // test_sig_match::test_hash();
    // address is USDC_WETH V3 contract https://etherscan.io/address/0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640
    let address = "0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640";

    let wrapped_json = std::fs::read_to_string("src/abi.json").unwrap();
    let abi: ethers::abi::Abi = serde_json::from_str(&wrapped_json).unwrap();

    // Fetch logs using the ABI
    if let Err(err) = fetch_eth_logs(address, &abi).await {
        eprintln!("Error: {}", err);
    }
}

