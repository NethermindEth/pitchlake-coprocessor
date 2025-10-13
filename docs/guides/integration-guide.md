# Integration Guide

Learn how to integrate the Pitchlake Coprocessor into your external application for verifiable gas fee option pricing.

## Table of Contents
- [Overview](#overview)
- [Integration Patterns](#integration-patterns)
- [Step-by-Step Integration](#step-by-step-integration)
- [Data Fetching](#data-fetching)
- [Proof Generation](#proof-generation)
- [Result Processing](#result-processing)
- [On-Chain Submission](#on-chain-submission)
- [Production Considerations](#production-considerations)

## Overview

The Pitchlake Coprocessor can be integrated as:

1. **Library Dependency** - Include as Rust crate
2. **Standalone Service** - Run as separate process/service
3. **CLI Tool** - Execute via command-line interface

This guide focuses on **library integration** for maximum flexibility.

## Integration Patterns

### Pattern 1: Direct Library Integration

**Best for:** Applications written in Rust

```rust
// Cargo.toml
[dependencies]
proof-composition-twap-maxreturn-reserveprice-floating-hashing-methods = { path = "../pitchlake-coprocessor/methods/proof-composition-twap-maxreturn-reserveprice-floating-hashing-methods" }
common = { path = "../pitchlake-coprocessor/common", features = ["original"] }
```

```rust
// Your application
use proof_composition::generate_proof;

fn main() {
    let gas_prices = fetch_gas_prices();
    let (receipt, output) = generate_proof(gas_prices).unwrap();
    process_results(output);
}
```

### Pattern 2: FFI Integration

**Best for:** Applications written in other languages

```rust
// Build C-compatible library
// Cargo.toml
[lib]
crate-type = ["cdylib"]

// lib.rs
#[no_mangle]
pub extern "C" fn generate_proof_ffi(
    data_ptr: *const f64,
    data_len: usize,
    out_ptr: *mut u8,
    out_len: *mut usize,
) -> i32 {
    // ... implementation
}
```

### Pattern 3: Service Integration

**Best for:** Microservices architecture

```rust
// HTTP API wrapper
use axum::Router;

async fn generate_proof_endpoint(
    Json(request): Json<ProofRequest>
) -> Json<ProofResponse> {
    let receipt = generate_proof(request.gas_prices).await?;
    Json(ProofResponse { receipt })
}
```

## Step-by-Step Integration

### Step 1: Set Up Project

```bash
# Create new Rust project
cargo new my-application
cd my-application

# Add dependencies
cat >> Cargo.toml << EOF
[dependencies]
common = { path = "../pitchlake-coprocessor/common", features = ["original"] }
core = { path = "../pitchlake-coprocessor/methods/core" }
hashing-felts-methods = { path = "../pitchlake-coprocessor/methods/hashing-felts-methods" }
max-return-floating-methods = { path = "../pitchlake-coprocessor/methods/max-return-floating-methods" }
twap-error-bound-floating-methods = { path = "../pitchlake-coprocessor/methods/twap-error-bound-floating-methods" }
remove-seasonality-error-bound-floating-methods = { path = "../pitchlake-coprocessor/methods/remove-seasonality-error-bound-floating-methods" }
add-twap-7d-error-bound-floating-methods = { path = "../pitchlake-coprocessor/methods/add-twap-7d-error-bound-floating-methods" }
calculate-pt-pt1-error-bound-floating-methods = { path = "../pitchlake-coprocessor/methods/calculate-pt-pt1-error-bound-floating-methods" }
simulate-price-verify-position-floating-methods = { path = "../pitchlake-coprocessor/methods/simulate-price-verify-position-floating-methods" }
proof-composition-twap-maxreturn-reserveprice-floating-hashing-methods = { path = "../pitchlake-coprocessor/methods/proof-composition-twap-maxreturn-reserveprice-floating-hashing-methods" }

risc0-zkvm = "2.3.2"
chrono = "0.4"
tokio = { version = "1", features = ["full"] }
EOF
```

### Step 2: Implement Data Fetching

```rust
// src/data_fetcher.rs

use chrono::{DateTime, Utc};
use std::error::Error;

/// Fetch historical gas fee data from Ethereum
pub async fn fetch_gas_fee_data(
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
) -> Result<Vec<f64>, Box<dyn Error>> {
    // Option 1: Fetch from Ethereum node
    fetch_from_node(start_time, end_time).await

    // Option 2: Fetch from indexer API
    // fetch_from_api(start_time, end_time).await

    // Option 3: Load from local database
    // fetch_from_database(start_time, end_time).await
}

async fn fetch_from_node(
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<f64>, Box<dyn Error>> {
    // Connect to Ethereum node
    let provider = Provider::<Http>::try_from("https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY")?;

    let mut gas_prices = Vec::new();
    let mut current = start;

    while current < end {
        // Fetch block at timestamp
        let block_number = find_block_at_timestamp(&provider, current.timestamp()).await?;
        let block = provider.get_block(block_number).await?;

        // Extract base fee
        let base_fee = block.base_fee_per_gas
            .ok_or("Block has no base fee")?;

        // Convert wei to gwei
        let base_fee_gwei = base_fee.as_u64() as f64 / 1e9;
        gas_prices.push(base_fee_gwei);

        // Move to next hour
        current += chrono::Duration::hours(1);
    }

    Ok(gas_prices)
}

async fn find_block_at_timestamp(
    provider: &Provider<Http>,
    target_timestamp: i64,
) -> Result<u64, Box<dyn Error>> {
    // Binary search to find block near timestamp
    // ... implementation ...
    Ok(block_number)
}
```

### Step 3: Implement Proof Generation

```rust
// src/proof_generator.rs

use core::{ProofCompositionInput, ProofCompositionOutput};
use common::original::calculate_reserve_price;
use hashing_felts_methods::hash_felts;
use max_return_floating_methods::max_return;
use twap_error_bound_floating_methods::calculate_twap;
// ... import other methods ...
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub fn generate_reserve_price_proof(
    gas_prices: Vec<f64>,
    start_timestamp: i64,
    end_timestamp: i64,
    strike_price: i64,
) -> Result<(Receipt, ProofCompositionOutput), Box<dyn std::error::Error>> {

    // Validate input
    validate_input(&gas_prices, start_timestamp, end_timestamp)?;

    // Step 1: Hash data
    let (hash_receipt, hash_result) = hash_felts(convert_to_felts(&gas_prices));

    // Step 2: Calculate max return
    let (max_ret_receipt, max_ret_result) = max_return(gas_prices.clone());

    // Step 3: Extract 3-month data
    let data_3_months = gas_prices[gas_prices.len().saturating_sub(2160)..].to_vec();

    // Step 4: Calculate TWAP
    let twap = common::floating_point::calculate_twap(&data_3_months);
    let (twap_receipt, _) = calculate_twap(data_3_months.clone(), twap, 1.0)?;

    // Step 5: Calculate reserve price (host computation)
    let reserve_result = calculate_reserve_price(
        &data_3_months,
        strike_price,
        720,  // n_periods
    );

    // Step 6: Verify seasonality removal
    let (seasonality_receipt, _) = remove_seasonality_error_bound(
        data_3_months.clone(),
        reserve_result.clone(),
    )?;

    // Step 7: Verify 7D TWAP
    let (twap_7d_receipt, _) = add_twap_7d_error_bound(
        data_3_months.clone(),
        reserve_result.twap_7d.clone(),
    )?;

    // Step 8: Verify Pt/Pt1 matrices
    let (pt_pt1_receipt, _) = calculate_pt_pt1_error_bound(
        reserve_result.clone(),
    )?;

    // Step 9: Simulate and verify
    let (simulate_receipt, _) = simulate_price_verify_position(
        reserve_result.clone(),
        4000,  // num_paths
    )?;

    // Step 10: Compose all proofs
    let composition_input = ProofCompositionInput {
        data_8_months: gas_prices,
        data_8_months_hash: hash_result.hash,
        start_timestamp,
        end_timestamp,
        twap_result: twap,
        max_return: max_ret_result.max_return,
        reserve_price: reserve_result.reserve_price,
        // ... fill other fields ...
    };

    let env = ExecutorEnv::builder()
        .add_assumption(hash_receipt)
        .add_assumption(twap_receipt)
        .add_assumption(max_ret_receipt)
        .add_assumption(seasonality_receipt)
        .add_assumption(twap_7d_receipt)
        .add_assumption(pt_pt1_receipt)
        .add_assumption(simulate_receipt)
        .write(&composition_input)?
        .build()?;

    let prove_info = default_prover().prove(
        env,
        COMPOSITION_GUEST_ELF,
    )?;

    let receipt = prove_info.receipt;

    // Verify proof
    receipt.verify(COMPOSITION_GUEST_ID)?;

    // Decode output
    let output: ProofCompositionOutput = receipt.journal.decode()?;

    Ok((receipt, output))
}

fn validate_input(
    data: &[f64],
    start: i64,
    end: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check data length
    if data.len() != 5760 {
        return Err(format!("Expected 5760 data points, got {}", data.len()).into());
    }

    // Check all values are positive
    for (i, &value) in data.iter().enumerate() {
        if value <= 0.0 {
            return Err(format!("Invalid gas price at index {}: {}", i, value).into());
        }
    }

    // Check timestamp range
    let duration_hours = (end - start) / 3600;
    if duration_hours < 2000 || duration_hours > 3000 {
        return Err("Timestamp range should be approximately 90 days".into());
    }

    Ok(())
}
```

### Step 4: Process Results

```rust
// src/result_processor.rs

use core::ProofCompositionOutput;
use common::fixed_point::UFixedPoint123x128;

pub struct ProcessedResults {
    pub reserve_price_gwei: f64,
    pub twap_gwei: f64,
    pub max_return_percent: f64,
    pub data_hash: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
}

pub fn process_output(
    output: ProofCompositionOutput
) -> Result<ProcessedResults, Box<dyn std::error::Error>> {

    // Decode hex strings to f64
    let reserve_price = decode_hex_to_f64(&output.reserve_price)?;
    let twap = decode_hex_to_f64(&output.twap_result)?;
    let max_return = decode_hex_to_f64(&output.max_return)?;

    // Format data hash
    let data_hash = format_data_hash(&output.data_8_months_hash);

    // Convert timestamps
    let start_time = chrono::DateTime::from_timestamp(output.start_timestamp, 0)
        .ok_or("Invalid start timestamp")?;
    let end_time = chrono::DateTime::from_timestamp(output.end_timestamp, 0)
        .ok_or("Invalid end timestamp")?;

    Ok(ProcessedResults {
        reserve_price_gwei: reserve_price,
        twap_gwei: twap,
        max_return_percent: max_return * 100.0,
        data_hash,
        start_time,
        end_time,
    })
}

fn decode_hex_to_f64(hex: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let fixed_point = UFixedPoint123x128::unpack_from_hex(hex)?;
    Ok(fixed_point.to_f64())
}

fn format_data_hash(hash: &[u32; 8]) -> String {
    hash.iter()
        .map(|x| format!("{:08x}", x))
        .collect::<Vec<_>>()
        .join("")
}
```

## Complete Example

```rust
// src/main.rs

mod data_fetcher;
mod proof_generator;
mod result_processor;

use chrono::{Duration, Utc};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 Pitchlake Coprocessor Integration Example");

    // Step 1: Define time period
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(90);
    let data_start_time = end_time - Duration::days(240);

    println!("📅 Time period: {} to {}", start_time, end_time);

    // Step 2: Fetch gas fee data
    println!("📊 Fetching gas fee data...");
    let gas_prices = data_fetcher::fetch_gas_fee_data(
        data_start_time,
        end_time
    ).await?;

    println!("✓ Fetched {} data points", gas_prices.len());

    // Step 3: Generate proof
    println!("🔐 Generating zero-knowledge proof...");
    println!("   (This may take 6-10 minutes)");

    let strike_price = 15000;  // 15 gwei

    let (receipt, output) = proof_generator::generate_reserve_price_proof(
        gas_prices,
        start_time.timestamp(),
        end_time.timestamp(),
        strike_price,
    )?;

    println!("✓ Proof generated successfully");

    // Step 4: Process results
    println!("📈 Processing results...");
    let results = result_processor::process_output(output)?;

    println!("\n═══════════════════════════════════════════");
    println!("           COMPUTATION RESULTS             ");
    println!("═══════════════════════════════════════════");
    println!("Reserve Price:  {:.2} gwei", results.reserve_price_gwei);
    println!("TWAP:           {:.2} gwei", results.twap_gwei);
    println!("Max Return:     {:.2}%", results.max_return_percent);
    println!("Data Hash:      {}", results.data_hash);
    println!("Period:         {} to {}", results.start_time, results.end_time);
    println!("═══════════════════════════════════════════\n");

    // Step 5: Save proof for on-chain submission
    println!("💾 Saving proof receipt...");
    let receipt_bytes = bincode::serialize(&receipt)?;
    std::fs::write("proof_receipt.bin", receipt_bytes)?;
    println!("✓ Proof saved to proof_receipt.bin");

    // Step 6: Submit to StarkNet (optional)
    if std::env::var("SUBMIT_TO_STARKNET").is_ok() {
        println!("📤 Submitting proof to StarkNet...");
        submit_to_starknet(&receipt)?;
        println!("✓ Proof submitted successfully");
    }

    Ok(())
}

fn submit_to_starknet(receipt: &risc0_zkvm::Receipt) -> Result<(), Box<dyn Error>> {
    // Implementation depends on your StarkNet setup
    // See OnChain-Integration-Summary.md for details
    todo!("Implement StarkNet submission")
}
```

## Production Considerations

### 1. Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProofGenerationError {
    #[error("Invalid input data: {0}")]
    InvalidInput(String),

    #[error("Data fetching failed: {0}")]
    DataFetchError(String),

    #[error("Proof generation failed: {0}")]
    ProofError(String),

    #[error("Verification failed")]
    VerificationError,
}
```

### 2. Performance Optimization

```rust
// Use Bonsai remote prover for faster generation
std::env::set_var("BONSAI_API_KEY", "your_key");

// Generate sub-proofs in parallel
use rayon::prelude::*;

let receipts: Vec<Receipt> = vec![
    || hash_felts(data),
    || max_return(data),
    || calculate_twap(data),
].par_iter()
 .map(|f| f().0)
 .collect();
```

### 3. Caching

```rust
// Cache generated proofs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CachedProof {
    data_hash: String,
    receipt: Receipt,
    output: ProofCompositionOutput,
    timestamp: i64,
}

fn cache_proof(proof: &CachedProof) -> Result<(), Box<dyn Error>> {
    let path = format!("cache/proof_{}.bin", proof.data_hash);
    let bytes = bincode::serialize(proof)?;
    std::fs::write(path, bytes)?;
    Ok(())
}
```

### 4. Monitoring

```rust
use tracing::{info, warn, error};

#[tracing::instrument]
async fn generate_proof_with_monitoring(
    data: Vec<f64>
) -> Result<Receipt, ProofGenerationError> {
    info!("Starting proof generation");

    let start = std::time::Instant::now();
    let result = generate_proof(data).await;
    let duration = start.elapsed();

    match &result {
        Ok(_) => info!("Proof generated in {:?}", duration),
        Err(e) => error!("Proof generation failed: {}", e),
    }

    result
}
```

## Next Steps

- [On-Chain Integration](../../OnChain-Integration-Summary.md) - Submit proofs to StarkNet
- [Environment Setup](environment-setup.md) - Configure environment variables
- [Running Methods](running-methods.md) - Test individual methods

## Support

For integration issues:
1. Check [Troubleshooting](../getting-started/quickstart.md#troubleshooting)
2. Review [Architecture Overview](../architecture/overview.md)
3. Open GitHub issue with integration details
