use ethers::{
    core::types::Log,
};
use crate::log_processing::to_hex;

pub fn pretty_print_log(log: &Log)  {
    println!("Address: {}", to_hex(&log.address.0)); // Assuming Address is H160 type
    println!("Topics:");
    for topic in &log.topics {
        println!("    {}", to_hex(&topic.0)); // Assuming topics are of H256 type
    }
    println!("Data: {}", to_hex(&log.data));
    if let Some(block_hash) = &log.block_hash {
        println!("Block Hash: {}", to_hex(&block_hash.0));
    }
    if let Some(block_number) = log.block_number {
        println!("Block Number: {}", block_number);
    }
    if let Some(transaction_hash) = &log.transaction_hash {
        println!("Transaction Hash: {}", to_hex(&transaction_hash.0));
    }
    if let Some(transaction_index) = log.transaction_index {
        println!("Transaction Index: {}", transaction_index);
    }
    if let Some(log_index) = log.log_index {
        println!("Log Index: {}", log_index);
    }
    if let Some(transaction_log_index) = log.transaction_log_index {
        println!("Transaction Log Index: {}", transaction_log_index);
    }
    if let Some(log_type) = &log.log_type {
        println!("Log Type: {}", log_type);
    }
    if let Some(removed) = log.removed {
        println!("Removed: {}", removed);
    }
}
