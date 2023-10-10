use std::collections::HashMap;
use reqwest;
use serde_json::Value;
use dotenv::dotenv;

use reqwest::blocking::Client as BlockingClient;

pub fn get_contract_abi(contract_address: &str) -> Result<Value, Box<dyn std::error::Error>> {
    dotenv().ok();
    let etherscan_api_key = std::env::var("ETHERSCAN_API_KEY").expect("ETHERSCAN_API_KEY not set");
    let etherscan_api_url = "https://api.etherscan.io/api"; // Adjust this if you have a different endpoint

    let mut params = HashMap::new();
    params.insert("module", "contract");
    params.insert("action", "getabi");
    params.insert("address", contract_address);
    params.insert("apikey", &etherscan_api_key);

    let client = BlockingClient::new();
    let response: Value = client.get(etherscan_api_url)
        .query(&params)
        .send()?
        .json()?;

    if response["status"] == "1" && response["result"].is_string() {
        let abi = serde_json::from_str(response["result"].as_str().unwrap())?;
        Ok(abi)
    } else {
        Err(format!("Error fetching ABI for {}. Error: {}", contract_address, response["message"].as_str().unwrap_or("Unknown error")).into())
    }
}
