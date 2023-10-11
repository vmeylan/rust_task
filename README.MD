# Ethereum Log Processor

This is a Rust-based Ethereum log processor that connects to the Ethereum network, fetches logs from a specific contract address, decodes these logs using an ABI (Application Binary Interface), and stores the decoded data in JSON format. This README provides an overview of the project and instructions for setting up and using the log processor.

## Table of Contents
- Overview
- Features
- Prerequisites
- Setup
- Usage
- Testing
- Contributing
- License

## Overview

The Ethereum Log Processor is a Rust application designed to fetch Ethereum logs from a specific contract address and decode these logs using the contract's ABI. It uses the `ethers` library for Ethereum interaction, which provides convenient tools for working with Ethereum data.

## Features

- Connects to the Ethereum network via Infura.
- Fetches Ethereum logs from a specific contract address.
- Decodes logs using the contract's ABI.
- Stores the decoded data in JSON format.

## Prerequisites

Before you can use the Ethereum Log Processor, make sure you have the following prerequisites installed:

- [Rust](https://www.rust-lang.org/): You'll need Rust and Cargo, its package manager, installed on your system.

## Setup

1. Clone the repository to your local machine:

```
git clone <repository-url>
cd task_rust
```

2. Create a `.env` file in the project's root directory and add your Infura API WebSocket key. You can obtain an API key by signing up at [Infura](https://infura.io/):
```
INFURA_API_KEY=<your-api-key>
```

3. Install the required Rust dependencies:
```
cargo build
```

## Usage

To use the Ethereum Log Processor, follow these steps:

1. Define the contract address and ABI: Modify the `main` function in `main.rs` to specify the Ethereum contract address and ABI. Ensure that the ABI is stored in a file named `abi.json` in the `src` directory.

```
let address = "<contract-address>"; // Replace with your contract address
let abi_json = std::fs::read_to_string("src/abi.json").unwrap();
let abi: ethers::abi::Abi = serde_json::from_str(&abi_json).unwrap();
```


2. Run the Ethereum Log Processor:

```cargo run```


The processor will connect to the Ethereum network using your Infura API WebSocket key, fetch logs from the specified contract address, decode the logs using the ABI, and store the decoded data in JSON files.

The decoded data will be stored in the `data` directory with filenames containing the contract address and the date of processing.

## Testing

The project includes unit tests to ensure the correctness of its components. To run the tests, use the following command:

```cargo test```

"""
