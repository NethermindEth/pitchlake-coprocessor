# Proof Composition Integration Guide

This guide explains how to call the proof composition system from an external application.

## Overview

The proof composition system generates a ZK proof that verifies gas fee option reserve price calculations. To use it, you need to:

1. Collect 8 months of historical gas fee data
2. Run preliminary computations (hashing, TWAP, reserve price, etc.)
3. Generate sub-proofs for each computation
4. Compose the final proof

## Architecture

```
External Application
    ↓
    ├─→ [1] Generate Sub-Proofs (in parallel)
    │   ├─ Hash gas fee data (Starknet field elements)
    │   ├─ Calculate max return (volatility)
    │   ├─ Calculate TWAP (time-weighted average)
    │   ├─ Remove seasonality (time series decomposition)
    │   ├─ Calculate 7-day TWAP
    │   ├─ Calculate Markov transition matrices (pt, pt_1)
    │   └─ Simulate prices and verify positions
    │
    ├─→ [2] Construct ProofCompositionInput
    │
    └─→ [3] Generate Composed Proof
```

## Data Requirements

### 1. Historical Gas Fee Data (8 months)

```rust
// Collect 5760 hours of hourly average gas fees (240 days × 24 hours)
// Format: Vec<f64> where each value is gas price in gwei
let data_8_months: Vec<f64> = vec![
    15.5,  // Hour 0
    16.2,  // Hour 1
    14.8,  // Hour 2
    // ... 5757 more values
];
```

**Where to get this data:**
- Ethereum node RPC: Query `eth_feeHistory` for each hour
- Etherscan API: Historical gas price data
- The Graph: Indexed blockchain data
- Internal database: If you already track gas prices

### 2. Time Periods

```rust
// Define your analysis periods
let now = chrono::Utc::now().timestamp();

// 8-month period (for max return calculation)
let data_8_months_start_timestamp = now - (3600 * 24 * 30 * 8); // 8 months ago
let data_8_months_end_timestamp = now;

// 3-month period (for reserve price calculation)
// This is the last 90 days of the 8-month dataset
let start_timestamp = now - (3600 * 24 * 90);  // 90 days ago
let end_timestamp = now;
```

## Step-by-Step Integration

### Step 1: Prepare Your Environment

```rust
use common::{
    original::{self, convert_array1_to_dvec},
    tests::mock::convert_data_to_vec_of_tuples,
};
use core::ProofCompositionInput;
use hashing_felts::hash_felts;
use max_return_floating::max_return;
use twap_error_bound_floating::calculate_twap;
use remove_seasonality_error_bound_floating::remove_seasonality_error_bound;
use add_twap_7d_error_bound_floating::add_twap_7d_error_bound;
use calculate_pt_pt1_error_bound_floating::calculate_pt_pt1_error_bound_floating;
use simulate_price_verify_position_floating::simulate_price_verify_position;
use risc0_zkvm::{default_prover, ExecutorEnv};
use proof_composition_twap_maxreturn_reserveprice_floating_hashing_methods::{
    PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_HASHING_GUEST_ELF,
    PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_HASHING_GUEST_ID,
};
```

### Step 2: Generate Sub-Proofs

```rust
pub fn generate_proof_composition(
    data_8_months: Vec<f64>,
    start_timestamp: i64,
    end_timestamp: i64,
) -> Result<Receipt, Box<dyn std::error::Error>> {

    // ═══════════════════════════════════════════════════════════
    // SUB-PROOF #1: Hash the 8-month data
    // ═══════════════════════════════════════════════════════════
    let inputs_felt = convert_f64_to_felts(data_8_months.clone());
    let (hashing_receipt, hashing_res) = hash_felts(HashingFeltInput {
        inputs: inputs_felt,
    });

    // ═══════════════════════════════════════════════════════════
    // SUB-PROOF #2: Calculate maximum return (volatility)
    // ═══════════════════════════════════════════════════════════
    let (max_return_receipt, max_return_res) = max_return(MaxReturnInput {
        data: data_8_months.clone(),
    });

    // ═══════════════════════════════════════════════════════════
    // Extract 3-month subset (last 2160 hours = 90 days)
    // ═══════════════════════════════════════════════════════════
    let data_3_months = data_8_months[data_8_months.len().saturating_sub(2160)..].to_vec();

    // ═══════════════════════════════════════════════════════════
    // SUB-PROOF #3: Calculate TWAP
    // ═══════════════════════════════════════════════════════════
    let twap_original = floating_point::calculate_twap(&data_3_months);
    let (calculate_twap_receipt, _) = calculate_twap(TwapErrorBoundInput {
        avg_hourly_gas_fee: data_3_months.clone(),
        twap_tolerance: 1.0,  // 1% tolerance
        twap_result: twap_original,
    });

    // ═══════════════════════════════════════════════════════════
    // Calculate Reserve Price (HOST COMPUTATION)
    // ═══════════════════════════════════════════════════════════
    let n_periods = 720;  // 720 three-hour periods in 90 days
    let strike_price = 15000;  // Strike price in gwei

    let data_with_timestamps = convert_data_to_vec_of_tuples(
        data_3_months.clone(),
        start_timestamp
    );

    // This is the expensive computation done on the host
    let res = original::calculate_reserve_price(
        &data_with_timestamps,
        strike_price,
        n_periods
    );

    // ═══════════════════════════════════════════════════════════
    // SUB-PROOF #4: Verify seasonality removal
    // ═══════════════════════════════════════════════════════════
    let floating_point_tolerance = 0.00001;  // 0.00001%
    let (remove_seasonality_receipt, _) = remove_seasonality_error_bound(
        RemoveSeasonalityErrorBoundFloatingInput {
            data: data_3_months.clone(),
            slope: res.slope,
            intercept: res.intercept,
            de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
                res.de_seasonalised_detrended_log_base_fee.clone()
            ),
            season_param: convert_array1_to_dvec(res.season_param.clone()),
            tolerance: floating_point_tolerance,
        }
    );

    // ═══════════════════════════════════════════════════════════
    // SUB-PROOF #5: Verify 7-day TWAP
    // ═══════════════════════════════════════════════════════════
    let (add_twap_7d_receipt, _) = add_twap_7d_error_bound(
        AddTwap7dErrorBoundFloatingInput {
            data: data_3_months.clone(),
            twap_7d: res.twap_7d.clone(),
            tolerance: floating_point_tolerance,
        }
    );

    // ═══════════════════════════════════════════════════════════
    // SUB-PROOF #6: Verify Markov transition matrices
    // ═══════════════════════════════════════════════════════════
    let (calculate_pt_pt1_receipt, _) = calculate_pt_pt1_error_bound_floating(
        CalculatePtPt1ErrorBoundFloatingInput {
            de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
                res.de_seasonalised_detrended_log_base_fee.clone()
            ),
            pt: convert_array1_to_dvec(res.pt.clone()),
            pt_1: convert_array1_to_dvec(res.pt_1.clone()),
            tolerance: floating_point_tolerance,
        }
    );

    // ═══════════════════════════════════════════════════════════
    // SUB-PROOF #7: Simulate prices and verify positions
    // ═══════════════════════════════════════════════════════════
    let num_paths = 4000;
    let gradient_tolerance = 5e-2;  // 5%
    let reserve_price_tolerance = 5.0;  // 5%

    let (simulate_price_receipt, _) = simulate_price_verify_position(
        SimulatePriceVerifyPositionInput {
            start_timestamp,
            end_timestamp,
            data_length: data_3_months.len(),
            positions: res.positions.clone(),
            pt: convert_array1_to_dvec(res.pt.clone()),
            pt_1: convert_array1_to_dvec(res.pt_1.clone()),
            gradient_tolerance,
            de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
                res.de_seasonalised_detrended_log_base_fee.clone()
            ),
            n_periods,
            num_paths,
            season_param: convert_array1_to_dvec(res.season_param.clone()),
            twap_7d: res.twap_7d.clone(),
            slope: res.slope,
            intercept: res.intercept,
            reserve_price: res.reserve_price,
            tolerance: reserve_price_tolerance,
        }
    );

    // ═══════════════════════════════════════════════════════════
    // COMPOSE: Build final input structure
    // ═══════════════════════════════════════════════════════════
    let data_8_months_start_timestamp = start_timestamp - (3600 * 24 * 30 * 5);

    let input = ProofCompositionInput {
        // Data and hash
        data_8_months_hash: hashing_res.hash,
        data_8_months: data_8_months.clone(),

        // 8-month period timestamps
        data_8_months_start_timestamp,
        data_8_months_end_timestamp: end_timestamp,

        // Overall timestamps (3-month period)
        start_timestamp,
        end_timestamp,

        // Specific calculation timestamps
        twap_start_timestamp: start_timestamp,
        twap_end_timestamp: end_timestamp,
        reserve_price_start_timestamp: start_timestamp,
        reserve_price_end_timestamp: end_timestamp,
        max_return_start_timestamp: data_8_months_start_timestamp,
        max_return_end_timestamp: end_timestamp,

        // Reserve price calculation results
        positions: res.positions,
        pt: convert_array1_to_dvec(res.pt),
        pt_1: convert_array1_to_dvec(res.pt_1),
        de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
            res.de_seasonalised_detrended_log_base_fee
        ),
        season_param: convert_array1_to_dvec(res.season_param),
        twap_7d: res.twap_7d,
        slope: res.slope,
        intercept: res.intercept,
        reserve_price: res.reserve_price,

        // Simulation parameters
        n_periods,
        num_paths,
        gradient_tolerance,

        // Tolerances
        floating_point_tolerance,
        reserve_price_tolerance,
        twap_tolerance: 1.0,

        // Results
        twap_result: twap_original,
        max_return: max_return_res.1,
    };

    // ═══════════════════════════════════════════════════════════
    // GENERATE: Create the composed proof
    // ═══════════════════════════════════════════════════════════
    let env = ExecutorEnv::builder()
        .add_assumption(hashing_receipt)
        .add_assumption(calculate_twap_receipt)
        .add_assumption(max_return_receipt)
        .add_assumption(remove_seasonality_receipt)
        .add_assumption(add_twap_7d_receipt)
        .add_assumption(calculate_pt_pt1_receipt)
        .add_assumption(simulate_price_receipt)
        .write(&input)?
        .build()?;

    let prove_info = default_prover().prove(
        env,
        PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_HASHING_GUEST_ELF,
    )?;

    let receipt = prove_info.receipt;

    // Verify the proof before returning
    receipt.verify(
        PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_HASHING_GUEST_ID
    )?;

    Ok(receipt)
}
```

### Step 3: Call from Your External Application

```rust
// In your external application
use your_proof_composition_crate::generate_proof_composition;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Fetch historical gas fee data
    let data_8_months = fetch_gas_fee_data_from_etherscan()?;

    // 2. Define time periods
    let now = chrono::Utc::now().timestamp();
    let start_timestamp = now - (3600 * 24 * 90);  // 90 days ago
    let end_timestamp = now;

    // 3. Generate the proof (this takes several minutes)
    println!("Generating ZK proof... (this may take 5-10 minutes)");
    let receipt = generate_proof_composition(
        data_8_months,
        start_timestamp,
        end_timestamp,
    )?;

    // 4. Extract the results from the proof
    let output: ProofCompositionOutput = receipt.journal.decode()?;

    println!("✅ Proof generated successfully!");
    println!("📊 Reserve Price: {}", output.reserve_price);
    println!("📊 TWAP: {}", output.twap_result);
    println!("📊 Max Return: {}", output.max_return);
    println!("🔗 Data Hash: {:?}", output.data_8_months_hash);

    // 5. Save the receipt for on-chain submission
    std::fs::write("proof_receipt.bin", bincode::serialize(&receipt)?)?;

    Ok(())
}

fn fetch_gas_fee_data_from_etherscan() -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    // Example: Fetch from Etherscan API
    let api_key = std::env::var("ETHERSCAN_API_KEY")?;
    let client = reqwest::blocking::Client::new();

    let mut data = Vec::new();
    let end_time = chrono::Utc::now().timestamp();
    let start_time = end_time - (3600 * 24 * 240); // 240 days

    // Fetch hourly data
    for hour_offset in 0..5760 {
        let timestamp = start_time + (hour_offset * 3600);

        // Query gas price for this hour
        let response: serde_json::Value = client
            .get("https://api.etherscan.io/api")
            .query(&[
                ("module", "gastracker"),
                ("action", "gasestimate"),
                ("gasprice", "2000000000"), // Example query
                ("apikey", &api_key),
            ])
            .send()?
            .json()?;

        let gas_price: f64 = response["result"].as_str()
            .unwrap_or("0")
            .parse()?;

        data.push(gas_price / 1e9); // Convert wei to gwei

        // Rate limiting
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    Ok(data)
}
```

## Field-by-Field Explanation

### Core Data Fields

| Field | Type | Description | How to Obtain |
|-------|------|-------------|---------------|
| `data_8_months` | `Vec<f64>` | 5760 hours of gas fees | Fetch from Ethereum node or API |
| `data_8_months_hash` | `[u32; 8]` | Hash commitment | Generated by `hash_felts()` |
| `data_8_months_start_timestamp` | `i64` | Start of 8-month period | Calculate: `end - (8 * 30 * 24 * 3600)` |
| `data_8_months_end_timestamp` | `i64` | End of 8-month period | Current timestamp or analysis end time |

### Time Period Fields

| Field | Type | Description | Typical Value |
|-------|------|-------------|---------------|
| `start_timestamp` | `i64` | Start of 3-month analysis | Last 90 days |
| `end_timestamp` | `i64` | End of 3-month analysis | Now or analysis end |
| `twap_start_timestamp` | `i64` | TWAP calculation start | Same as `start_timestamp` |
| `twap_end_timestamp` | `i64` | TWAP calculation end | Same as `end_timestamp` |
| `reserve_price_start_timestamp` | `i64` | Reserve price calc start | Same as `start_timestamp` |
| `reserve_price_end_timestamp` | `i64` | Reserve price calc end | Same as `end_timestamp` |
| `max_return_start_timestamp` | `i64` | Max return calc start | `data_8_months_start_timestamp` |
| `max_return_end_timestamp` | `i64` | Max return calc end | `data_8_months_end_timestamp` |

### Reserve Price Calculation Fields

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `positions` | `Vec<f64>` | Option positions (delta, vega, theta) | `res.positions` from `calculate_reserve_price()` |
| `pt` | `DVector<f64>` | Markov transition matrix t | `res.pt` from `calculate_reserve_price()` |
| `pt_1` | `DVector<f64>` | Markov transition matrix t-1 | `res.pt_1` from `calculate_reserve_price()` |
| `de_seasonalised_detrended_log_base_fee` | `DVector<f64>` | Time series residuals | `res.de_seasonalised_detrended_log_base_fee` |
| `season_param` | `DVector<f64>` | 24 hourly seasonal params | `res.season_param` |
| `twap_7d` | `Vec<f64>` | 7-day rolling TWAP | `res.twap_7d` |
| `slope` | `f64` | Linear trend slope | `res.slope` |
| `intercept` | `f64` | Linear trend intercept | `res.intercept` |
| `reserve_price` | `f64` | Calculated reserve price | `res.reserve_price` |

### Simulation Parameters

| Field | Type | Description | Recommended Value |
|-------|------|-------------|-------------------|
| `n_periods` | `usize` | Number of 3-hour periods | 720 (for 90 days) |
| `num_paths` | `usize` | Monte Carlo paths | 4000 |

### Tolerance Fields

| Field | Type | Description | Recommended Value |
|-------|------|-------------|-------------------|
| `gradient_tolerance` | `f64` | Convergence tolerance | 0.05 (5%) |
| `floating_point_tolerance` | `f64` | Arithmetic precision | 0.00001 (0.00001%) |
| `reserve_price_tolerance` | `f64` | Reserve price tolerance | 5.0 (5%) |
| `twap_tolerance` | `f64` | TWAP tolerance | 1.0 (1%) |

### Result Fields

| Field | Type | Description | Source |
|-------|------|-------------|--------|
| `twap_result` | `f64` | Calculated TWAP | From `floating_point::calculate_twap()` |
| `max_return` | `f64` | Maximum return | From `max_return()` function |

## Performance Considerations

### Execution Time

- **Host computation** (reserve price): 1-2 minutes
- **Sub-proof generation**: 3-5 minutes (parallelizable)
- **Proof composition**: 2-3 minutes
- **Total**: 6-10 minutes

### Parallelization

You can generate sub-proofs in parallel:

```rust
use rayon::prelude::*;

// Generate sub-proofs in parallel
let (hashing_receipt, max_return_receipt, twap_receipt) = rayon::join(
    || hash_felts(hashing_input),
    || max_return(max_return_input),
    || calculate_twap(twap_input),
);
```

### Memory Requirements

- **RAM**: 8-16 GB recommended
- **Disk**: 500 MB for proof artifacts

## Testing

### Mock Data for Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use common::tests::mock::get_5760_avg_base_fees_felt;

    #[test]
    fn test_proof_composition() {
        // Use mock data
        let data_8_months = get_mock_gas_fee_data();
        let start_timestamp = 1708833600;
        let end_timestamp = start_timestamp + (3600 * 24 * 90);

        // Generate proof
        let receipt = generate_proof_composition(
            data_8_months,
            start_timestamp,
            end_timestamp,
        ).unwrap();

        // Verify output
        let output: ProofCompositionOutput = receipt.journal.decode().unwrap();
        assert!(output.reserve_price > 0.0);
        assert!(output.twap_result > 0.0);
    }

    fn get_mock_gas_fee_data() -> Vec<f64> {
        // Generate 5760 hours of mock data
        (0..5760)
            .map(|i| {
                // Simulate daily patterns
                let hour_of_day = i % 24;
                let base = 15.0;
                let daily_variation = (hour_of_day as f64 * 0.5).sin() * 3.0;
                base + daily_variation
            })
            .collect()
    }
}
```

## Common Issues & Solutions

### Issue 1: "Data length mismatch"

**Cause**: Not exactly 5760 data points

**Solution**: Ensure you have exactly 5760 hourly values (240 days × 24 hours)

```rust
assert_eq!(data_8_months.len(), 5760, "Must have exactly 5760 data points");
```

### Issue 2: "Proof verification failed"

**Cause**: Tolerances too tight or data quality issues

**Solution**: Increase tolerances or validate input data:

```rust
// Validate data
for (i, &value) in data_8_months.iter().enumerate() {
    assert!(value > 0.0, "Gas fee at hour {} is non-positive", i);
    assert!(value < 10000.0, "Gas fee at hour {} is unrealistically high", i);
}
```

### Issue 3: "Out of memory"

**Cause**: Insufficient RAM for proof generation

**Solution**: Use a machine with more RAM or reduce `num_paths`:

```rust
// Reduce from 4000 to 2000 for lower memory usage
let num_paths = 2000;
```

## Next Steps

1. **Integrate with your data source**: Replace mock data with real Ethereum gas prices
2. **Set up a proving service**: Run proof generation on a dedicated server
3. **Deploy on-chain verifier**: Deploy the RISC Zero verifier contract on Starknet
4. **Automate**: Set up periodic proof generation (e.g., daily or weekly)

## Questions?

Common questions and their answers:

**Q: Can I use different time periods?**
A: Yes, but maintain the ratio: 8 months for max return, last 90 days for reserve price.

**Q: How often should I generate new proofs?**
A: Depends on your use case. Daily or weekly is common for option pricing.

**Q: Can I skip sub-proofs I don't need?**
A: No, all 7 sub-proofs are required for the composition to work.

**Q: How do I submit the proof on-chain?**
A: Use the receipt bytes and submit to the RISC Zero verifier contract on Starknet.
