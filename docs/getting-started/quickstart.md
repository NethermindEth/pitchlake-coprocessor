# Quick Start Guide

Get up and running with the Pitchlake Coprocessor in 5 minutes.

## Table of Contents
- [Prerequisites](#prerequisites)
- [Run Your First Proof](#run-your-first-proof)
- [Understanding the Output](#understanding-the-output)
- [Next Steps](#next-steps)

## Prerequisites

Before starting, ensure you have:

1. **Installed dependencies** - See [Installation Guide](installation.md)
2. **Cloned the repository**
3. **Built the project** - `cargo build --release`

## Run Your First Proof

### Option 1: Mock Proof (Fastest - ~1 second)

The mock proof composition runs all calculations without generating actual zero-knowledge proofs. Perfect for testing and development.

```bash
# Navigate to mock proof composition
cd /path/to/pitchlake-coprocessor/mains/mock-proof-composition

# Run with dev mode enabled
RISC0_DEV_MODE=1 cargo run --release
```

**Expected output:**
```
Loading data...
Data loaded: 5760 points
Calculating reserve price...
Reserve price calculated: 15234.56
Generating mock proof...
✓ Mock proof generated successfully

ProofCompositionOutput {
    data_8_months_hash: [0x1a2b3c4d, 0x5e6f7a8b, ...],
    start_timestamp: 1708833600,
    end_timestamp: 1716609600,
    reserve_price: "0x3ba5f3c8d2e1f0a9",
    twap_result: "0x2a1b4c5d6e7f8a9b",
    max_return: "0x4d5e6f7a8b9c0d1e",
    ...
}
```

### Option 2: Full Proof (Production - ~6-10 minutes)

Generate a real zero-knowledge proof that can be verified on-chain.

```bash
# Navigate to full proof composition
cd /path/to/pitchlake-coprocessor/mains/proof-composition-twap-maxreturn-reserveprice-floating-hashing

# Run with dev mode disabled (generates real proofs)
RISC0_DEV_MODE=0 cargo run --release
```

**This will:**
1. Load 5760 hours of historical gas fee data
2. Generate 7 sub-proofs (hashing, max return, TWAP, seasonality, etc.)
3. Compose sub-proofs into a single verifiable proof
4. Output the proof receipt for on-chain submission

**Performance note:** First run takes longer due to compilation. Subsequent runs are faster.

## Understanding the Output

### ProofCompositionOutput Structure

```rust
pub struct ProofCompositionOutput {
    pub data_8_months_hash: [u32; 8],      // Hash of input data
    pub start_timestamp: i64,               // Start of 90-day period
    pub end_timestamp: i64,                 // End of 90-day period
    pub reserve_price: String,              // Hex-encoded reserve price
    pub twap_result: String,                // Hex-encoded TWAP
    pub max_return: String,                 // Hex-encoded max return
    pub floating_point_tolerance: String,   // Tolerance for FP calculations
    pub reserve_price_tolerance: String,    // Tolerance for reserve price
    pub twap_tolerance: String,             // Tolerance for TWAP
    pub gradient_tolerance: String,         // Tolerance for gradients
}
```

### Key Metrics

1. **`reserve_price`** - The calculated option strike price based on:
   - Historical gas fee patterns
   - Seasonal adjustments
   - Monte Carlo simulation (4000 paths)
   - Markov transition matrices

2. **`twap_result`** - Time-weighted average price over 90 days
   - Fair pricing metric for options
   - Weighted by time spent at each price level

3. **`max_return`** - Maximum return over 240-day period
   - Volatility measure
   - Risk assessment metric

### Hex-Encoded Values

Numeric values are hex-encoded using fixed-point representation (`UFixedPoint123x128`):

```rust
// Example: Decode reserve price
let reserve_price_hex = "0x3ba5f3c8d2e1f0a9";
let reserve_price_f64 = UFixedPoint123x128::unpack_from_hex(reserve_price_hex);
println!("Reserve Price: {} gwei", reserve_price_f64);
```

## Verify the Proof

### Local Verification (Included in output)

The proof is automatically verified during generation:

```
✓ Proof generated successfully
✓ Proof verified locally
✓ Journal decoded successfully
```

### On-Chain Verification

To verify the proof on StarkNet:

1. **Deploy verifier contract** (if not already deployed):
   ```bash
   cd pitchlake_verifier
   scarb build
   starkli deploy --network sepolia
   ```

2. **Submit proof to contract**:
   ```bash
   # Use the generated proof receipt
   starkli invoke <verifier-address> verify_proof <proof-bytes>
   ```

See [On-Chain Integration](../../OnChain-Integration-Summary.md) for details.

## Data File Setup

Some methods require `data.csv` (historical gas fee data):

```bash
# Extract compressed data file
gunzip -c data.csv.gz > data.csv

# Methods that need data.csv in their directory:
# - twap-error-bound-floating
# - proof-composition-twap-maxreturn-reserveprice-floating-hashing

# Copy to method directory if needed
cp data.csv mains/twap-error-bound-floating/
```

**Data format:**
- 5760 rows (240 days × 24 hours)
- Each row: timestamp, hourly average gas price in gwei

## Common Use Cases

### 1. Fast Development Iteration

```bash
# Use mock proof for quick testing
cd mains/mock-proof-composition
RISC0_DEV_MODE=1 cargo run --release

# Make code changes
# Test again (fast)
RISC0_DEV_MODE=1 cargo run --release
```

### 2. Test Individual Sub-Proofs

```bash
# Test only TWAP calculation
cd mains/twap-error-bound-floating
RISC0_DEV_MODE=1 cargo run --release

# Test only max return
cd mains/max-return-floating
RISC0_DEV_MODE=1 cargo run --release
```

### 3. Generate Production Proof

```bash
# Full proof with real proving
cd mains/proof-composition-twap-maxreturn-reserveprice-floating-hashing
RISC0_DEV_MODE=0 cargo run --release

# Save proof receipt for on-chain submission
# (receipt is saved automatically by the program)
```

### 4. Benchmark Performance

```bash
# Measure proof generation time
time RISC0_DEV_MODE=0 cargo run --release
```

## Environment Variables

### RISC0_DEV_MODE

Controls whether real proofs are generated:

```bash
# Dev mode: No actual proving, fast execution (~1-2 minutes)
export RISC0_DEV_MODE=1

# Production mode: Real proofs, slow execution (~6-10 minutes)
export RISC0_DEV_MODE=0
```

### BONSAI_API_KEY (Optional)

Use Bonsai remote prover for faster proof generation:

```bash
# Sign up at https://bonsai.xyz and get API key
export BONSAI_API_KEY="your_api_key_here"

# Bonsai is automatically used when key is set
RISC0_DEV_MODE=0 cargo run --release
```

### RUST_LOG (Optional)

Control logging verbosity:

```bash
# Show debug logs
export RUST_LOG=debug

# Show only warnings and errors
export RUST_LOG=warn
```

## Troubleshooting

### Issue: "data.csv not found"

```bash
# Extract data file
gunzip -c data.csv.gz > data.csv

# Copy to method directory
cp data.csv mains/your-method/
```

### Issue: "Out of memory"

```bash
# Reduce parallel jobs
export CARGO_BUILD_JOBS=2

# Use dev mode (less memory intensive)
export RISC0_DEV_MODE=1
```

### Issue: "Proof verification failed"

- Check input data has exactly 5760 values
- Verify all gas prices are positive
- Check timestamp ranges are valid
- Try increasing tolerance values

### Issue: Slow compilation

```bash
# First build is always slow (10-15 minutes)
# Subsequent builds are faster (~2-3 minutes)

# Use incremental compilation (default)
# Avoid `cargo clean` unless necessary
```

## Next Steps

Now that you've run your first proof, explore:

1. **[Testing Guide](testing.md)** - Run tests and write new ones
2. **[Running Methods](../guides/running-methods.md)** - Execute individual methods
3. **[Integration Guide](../guides/integration-guide.md)** - Integrate with your application
4. **[Architecture Overview](../architecture/overview.md)** - Understand the system design

## Quick Reference

```bash
# Mock proof (fast)
cd mains/mock-proof-composition
RISC0_DEV_MODE=1 cargo run --release

# Full proof (slow, production)
cd mains/proof-composition-twap-maxreturn-reserveprice-floating-hashing
RISC0_DEV_MODE=0 cargo run --release

# Individual method
cd mains/twap-error-bound-floating
RISC0_DEV_MODE=1 cargo run --release

# Run tests
cargo test --all

# Build all methods
cargo build --release --all
```
