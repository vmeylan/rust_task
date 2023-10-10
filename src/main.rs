use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize)]
struct EthLogRequest {
    jsonrpc: &'static str,
    method: &'static str,
    params: Vec<LogParams>,
    id: u64,
}

#[derive(Serialize)]
struct LogParams {
    topics: Vec<String>,
}

#[tokio::main]
async fn fetch_eth_logs(address: &str, id: u64) -> Result<(), reqwest::Error> {
    let endpoint = "https://mainnet.infura.io/v3/YOUR_INFURA_PROJECT_ID"; // Replace with your Infura Project ID

    let req = EthLogRequest {
        jsonrpc: "2.0",
        method: "eth_getLogs",
        params: vec![LogParams {
            topics: vec![format!("0x000000000000000000000000{}", address)],
        }],
        id,
    };

    let client = reqwest::Client::new();
    let res = client.post(endpoint)
        .header("Content-Type", "application/json")
        .body(json!(req).to_string())
        .send()
        .await?;

    // Print the response for demonstration purposes
    println!("{:#?}", res.text().await?);

    Ok(())
}

fn main() {
    let address = "a94f5374fce5edbc8e2a8697c15331677e6ebf0b"; // replace with the address of interest
    let id = 74; // replace with the desired ID
    let _ = fetch_eth_logs(address, id);
}