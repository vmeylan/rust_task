use ethers::{
    core::types::Log,
};
use crate::log_processing::to_hex;
use std::env;
use std::path::{PathBuf};

pub fn root_dir() -> Option<String> {
    // Get the current directory
    let current_dir = env::current_dir().ok()?;

    // Define the path to the root directory
    let mut root_dir = PathBuf::from(current_dir);

    // Iterate upwards from the current directory to find the root directory
    while !root_dir.join(".git").exists() {
        // If we reach the filesystem root without finding .git, return None
        if !root_dir.pop() {
            return None;
        }
    }

    Some(root_dir.to_string_lossy().into_owned())
}

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
