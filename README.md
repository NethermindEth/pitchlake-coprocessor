# Pitchlake Coprocessor

This repository implements the **Pitchlake Coprocessor**, a zero-knowledge proof system for verifiable financial computation using RISC Zero zkVM. The coprocessor generates cryptographic proofs for gas fee option pricing calculations (TWAP, reserve price, max return) used in the Pitchlake options market on StarkNet.

## Overview

The Pitchlake Coprocessor performs verifiable computations on Ethereum Layer 1 gas fee data to calculate critical financial metrics for the Pitchlake options protocol. It uses RISC Zero's zero-knowledge virtual machine (zkVM) to generate proofs that can be verified on-chain via StarkNet smart contracts.

### Key Features

- **Zero-Knowledge Proof Generation** - RISC Zero zkVM proofs for financial calculations
- **Proof Composition** - Seven specialized sub-proofs composed into a single verifiable proof
- **On-Chain Verification** - Groth16 proof verification on StarkNet via Cairo contracts
- **Floating-Point Accuracy** - Configurable error bounds for financial precision
- **Modular Architecture** - Independent methods for testing and development
- **Production-Ready** - Optimized for integration with external systems

### What It Computes

The coprocessor calculates three critical financial metrics:

1. **Reserve Price** - Option strike price calculated from historical gas fee patterns, seasonality, and Monte Carlo simulation
2. **TWAP (Time-Weighted Average Price)** - 90-day weighted average of gas fees for fair pricing
3. **Max Return** - Maximum return over 240-day period for risk assessment

## Documentation

### 📚 Getting Started
- [Installation Guide](docs/getting-started/installation.md) - Set up your development environment
- [Quick Start](docs/getting-started/quickstart.md) - Run your first proof in 5 minutes
- [Testing Guide](docs/getting-started/testing.md) - Run tests and benchmarks

### 🏗️ Architecture
- [Architecture Overview](docs/architecture/overview.md) - High-level system design
- [Proof Composition](docs/architecture/proof-composition.md) - How seven sub-proofs compose into final proof
- [Methods Overview](docs/architecture/methods-overview.md) - Detailed breakdown of each zkVM method

### 📖 Guides
- [Environment Setup](docs/guides/environment-setup.md) - Configure environment and data files
- [Running Methods](docs/guides/running-methods.md) - Execute individual methods and full composition
- [Integration Guide](docs/guides/integration-guide.md) - Integrate with your application
- [On-Chain Integration](OnChain-Integration-Summary.md) - Deploy and verify proofs on StarkNet

### 📦 Methods
- [Hashing Felts](docs/methods/hashing-felts.md) - Hash 8-month gas fee data as StarkNet field elements
- [Max Return](docs/methods/max-return.md) - Calculate maximum return (volatility measure)
- [TWAP](docs/methods/twap.md) - Time-weighted average price calculation
- [Remove Seasonality](docs/methods/remove-seasonality.md) - Detrend and deseasonalize time series
- [Add TWAP 7D](docs/methods/add-twap-7d.md) - Verify 7-day rolling TWAP
- [Calculate Pt Pt1](docs/methods/calculate-pt-pt1.md) - Verify Markov transition matrices
- [Simulate Price](docs/methods/simulate-price.md) - Monte Carlo simulation and position verification

### 📜 Contracts
- [Pitchlake Verifier](docs/contracts/pitchlake-verifier.md) - RISC Zero Groth16 proof verification on StarkNet
- [Mock Contracts](docs/contracts/mocks.md) - Testing contracts for development

### 📋 Reference
- [ProofCompositionOutput](ProofCompositionOutput.md) - Output structure and field descriptions
- [ProofCompositionInput](PROOF_COMPOSITION_GUIDE.md#proof-composition-integration-guide) - Input structure and data requirements

## Quick Start

### 1. Installation

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install RISC Zero toolchain
curl -L https://risczero.com/install | bash
rzup install

# Install StarkNet tools (via asdf)
asdf plugin add scarb && asdf plugin add starknet-foundry
asdf install

# Build all methods
cargo build --release
```

### 2. Prepare Data

```bash
# Extract historical gas fee data (5760 hours = 240 days)
gunzip -c data.csv.gz > data.csv

# Place data.csv in the method directory you want to run
# (Only needed for certain methods that load from CSV)
```

### 3. Run Mock Proof (Fast)

```bash
# Run mock proof composition (no actual proving, ~1 second)
cd mains/mock-proof-composition
RISC0_DEV_MODE=1 cargo run --release

# Output: ProofCompositionOutput with reserve_price, twap_result, max_return
```

### 4. Run Full Proof (Production)

```bash
# Run full proof composition with real proving (~6-10 minutes)
cd mains/proof-composition-twap-maxreturn-reserveprice-floating-hashing
RISC0_DEV_MODE=0 cargo run --release

# Generates verifiable proof for on-chain submission
```

## Environment Configuration

### Development Mode

For fast testing without proof generation:

```bash
export RISC0_DEV_MODE=1
```

### Production Mode

For generating real proofs:

```bash
export RISC0_DEV_MODE=0
export BONSAI_API_KEY="your_bonsai_api_key"  # Optional: Use Bonsai remote prover
```

## Project Structure

```
pitchlake-coprocessor/
├── docs/                          # Complete documentation
│   ├── getting-started/           # Installation, quickstart, testing
│   ├── architecture/              # System design and proof composition
│   ├── guides/                    # Operational guides
│   ├── methods/                   # Method-specific documentation
│   └── contracts/                 # Smart contract documentation
├── common/                        # Shared Rust libraries
│   └── src/
│       ├── fixed_point/          # Fixed-point arithmetic
│       ├── floating_point/       # Floating-point calculations
│       └── original/             # Core financial algorithms
├── methods/                       # RISC Zero zkVM guest programs
│   ├── core/                     # Shared types and utilities
│   ├── hashing-felts-methods/    # Data hashing
│   ├── max-return-floating-methods/      # Max return calculation
│   ├── twap-error-bound-floating-methods/# TWAP calculation
│   ├── remove-seasonality-error-bound-floating-methods/
│   ├── add-twap-7d-error-bound-floating-methods/
│   ├── calculate-pt-pt1-error-bound-floating-methods/
│   ├── simulate-price-verify-position-floating-methods/
│   ├── proof-composition-twap-maxreturn-reserveprice-floating-hashing-methods/
│   └── mock-proof-composition/   # Fast mock for testing
├── mains/                         # Host programs (call guest programs)
│   ├── hashing-felts/
│   ├── max-return-floating/
│   ├── twap-error-bound-floating/
│   ├── remove-seasonality-error-bound-floating/
│   ├── add-twap-7d-error-bound-floating/
│   ├── calculate-pt-pt1-error-bound-floating/
│   ├── simulate-price-verify-position-floating/
│   ├── proof-composition-twap-maxreturn-reserveprice-floating-hashing/
│   └── mock-proof-composition/
├── pitchlake_verifier/            # StarkNet Cairo contracts
│   ├── src/
│   │   ├── pitchlake_verifier.cairo  # RISC Zero proof verifier
│   │   └── mocks/                    # Test contracts
│   └── tests/                        # Contract tests
├── data.csv.gz                    # Historical gas fee data (compressed)
├── Cargo.toml                     # Workspace configuration
└── Scarb.toml                     # Cairo project configuration
```

## How It Works

### Proof Composition Architecture

The Pitchlake Coprocessor uses a **recursive proof composition** architecture where seven specialized sub-proofs are combined into a single verifiable proof:

```
External Application
    │
    ├─ Historical Gas Fee Data (5760 hours)
    │
    ▼
┌───────────────────────────────────────────────────────────┐
│                   SUB-PROOFS (Parallel)                   │
├───────────────────────────────────────────────────────────┤
│ [1] Hash Gas Fee Data         → data_8_months_hash        │
│ [2] Calculate Max Return       → max_return               │
│ [3] Calculate TWAP             → twap_result              │
│ [4] Remove Seasonality         → Verified ✓               │
│ [5] Calculate 7D TWAP          → Verified ✓               │
│ [6] Calculate Pt/Pt1 Matrices  → Verified ✓               │
│ [7] Simulate & Verify Position → reserve_price ✓          │
└───────────────────────────────────────────────────────────┘
    │
    ▼
┌───────────────────────────────────────────────────────────┐
│          PROOF COMPOSITION (Single zkVM Proof)            │
├───────────────────────────────────────────────────────────┤
│  • Combines all 7 sub-proof receipts                      │
│  • Verifies internal consistency                          │
│  • Produces final ProofCompositionOutput                  │
└───────────────────────────────────────────────────────────┘
    │
    ▼
┌───────────────────────────────────────────────────────────┐
│           ON-CHAIN VERIFICATION (StarkNet)                │
├───────────────────────────────────────────────────────────┤
│  • Groth16 proof verification via Cairo contract          │
│  • Decodes journal to ProofCompositionOutput              │
│  • Sends results to Pitchlake vault contracts             │
└───────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Input**: 5760 hours (240 days) of Ethereum L1 gas fee data
2. **Processing**: Seven specialized zkVM methods each generate a proof
3. **Composition**: Sub-proofs are composed into a single proof using RISC Zero's composition feature
4. **Output**: `ProofCompositionOutput` containing reserve price, TWAP, max return, and metadata
5. **Verification**: Proof submitted to StarkNet for on-chain verification

### Why Proof Composition?

- **Modularity** - Each method can be tested independently
- **Efficiency** - Sub-proofs can be generated in parallel
- **Maintainability** - Individual methods can be updated without affecting others
- **Reusability** - Sub-proofs can be reused across different compositions

## Integration Example

### From Your External Application

```rust
use common::original::calculate_reserve_price;
use proof_composition_twap_maxreturn_reserveprice_floating_hashing::generate_proof;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Fetch historical gas fee data (5760 hours)
    let gas_prices: Vec<f64> = fetch_gas_fee_data_from_ethereum()?;

    // 2. Define time period (90 days for reserve price)
    let now = chrono::Utc::now().timestamp();
    let start = now - (90 * 24 * 3600);
    let end = now;

    // 3. Generate proof (takes 6-10 minutes)
    let receipt = generate_proof(gas_prices, start, end)?;

    // 4. Extract results
    let output: ProofCompositionOutput = receipt.journal.decode()?;
    println!("Reserve Price: {}", output.reserve_price);
    println!("TWAP: {}", output.twap_result);
    println!("Max Return: {}", output.max_return);

    // 5. Submit proof to StarkNet
    submit_to_starknet(receipt)?;

    Ok(())
}
```

See [Integration Guide](docs/guides/integration-guide.md) for complete examples.

## Testing

### Run All Tests

```bash
cargo test --all
```

### Run Individual Method Tests

```bash
# Test TWAP calculation
cargo test -p twap-error-bound-floating-methods

# Test max return calculation
cargo test -p max-return-floating-methods
```

### Run Mock Proof for Fast Iteration

```bash
cd mains/mock-proof-composition
RISC0_DEV_MODE=1 cargo run --release
```

### Benchmarking

```bash
# Run with timing information
RISC0_DEV_MODE=1 cargo run --release -- --benchmark
```

## Technology Stack

**Zero-Knowledge Proofs:**
- RISC Zero zkVM 2.3.2 - Zero-knowledge virtual machine
- Groth16 - Proof verification scheme for StarkNet

**Language & Runtime:**
- Rust (stable) - Host programs and shared libraries
- Cargo - Build system and package manager

**Blockchain:**
- StarkNet - Layer 2 blockchain for proof verification
- Cairo 2.x - Smart contract language
- Scarb - Cairo package manager
- Starknet Foundry - Cairo testing framework

**Scientific Computing:**
- nalgebra - Linear algebra
- statrs - Statistical distributions
- polars - Data frame operations (optional)
- ndarray - N-dimensional arrays (optional)

## Performance Considerations

### Proof Generation Time

| Method | Dev Mode | Production Mode |
|--------|----------|-----------------|
| Mock Proof Composition | ~1 second | N/A |
| Individual Sub-Proof | ~30-60 seconds | ~2-3 minutes |
| Full Proof Composition | ~2 minutes | ~6-10 minutes |

### Memory Requirements

- **RAM**: 8-16 GB recommended for proof generation
- **Disk**: 500 MB for proof artifacts and build cache
- **CPU**: 4+ cores recommended (sub-proofs can run in parallel)

### Optimization Tips

1. **Use Dev Mode for Testing**: Set `RISC0_DEV_MODE=1` for fast iteration
2. **Parallelize Sub-Proofs**: Generate sub-proofs concurrently when possible
3. **Use Bonsai Prover**: Offload proving to Bonsai API for faster generation
4. **Optimize Data Loading**: Load `data.csv` once and reuse across methods

## On-Chain Gas Optimization

The current `ProofCompositionOutput` includes 10 fields (~280 bytes). For gas optimization on StarkNet, consider using the minimal structure with only 5 essential fields (~144 bytes):

```cairo
struct MinimalJournal {
    start_timestamp: u64,      // Time bounds
    end_timestamp: u64,        // Time bounds
    reserve_price: felt252,    // Primary output
    twap_result: felt252,      // Key metric
    max_return: felt252,       // Risk metric
}
```

This reduces gas costs by ~48% while maintaining all business-critical functionality. The zero-knowledge proof already guarantees computational correctness, making tolerance fields redundant on-chain.

See [On-Chain Integration Summary](OnChain-Integration-Summary.md) for details.

## Troubleshooting

### Common Issues

**Issue**: `cargo: command not found`
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Issue**: `cargo risczero: command not found`
```bash
# Install RISC Zero toolchain
curl -L https://risczero.com/install | bash
$HOME/.risc0/bin/rzup install
```

**Issue**: `data.csv not found`
```bash
# Extract data file
gunzip -c data.csv.gz > data.csv
# Copy to method directory if needed
cp data.csv mains/your-method/
```

**Issue**: Out of memory during proof generation
```bash
# Reduce parallel jobs
export CARGO_BUILD_JOBS=2
# Use dev mode for testing
export RISC0_DEV_MODE=1
```

**Issue**: Proof verification failed
- Check that input data has exactly 5760 values (240 days × 24 hours)
- Verify all gas prices are positive (> 0)
- Increase tolerance values if precision errors occur
- Validate timestamp ranges are correct

## Future Considerations

### Bonsai Prover Deprecation

RISC Zero has announced plans to deprecate the Bonsai remote prover in favor of **Boundless**, a decentralized proving marketplace. Future maintainers should:

1. **Migrate to Boundless** - Update proof submission workflows for decentralized proving
2. **Evaluate SP1** - Consider migrating to SP1 (Succinct) for improved performance on large datasets
3. **Optimize Proof Format** - Review proof size and verification costs for alternative zkVM systems

See [Architecture Overview](docs/architecture/overview.md#future-considerations) for detailed analysis.

## Contributing

### Development Workflow

```bash
# 1. Make changes to code
# 2. Run tests
cargo test --all

# 3. Run clippy for linting
cargo clippy --all-targets --all-features

# 4. Format code
cargo fmt --all

# 5. Build release
cargo build --release
```

### Code Standards

- **Rust**: Follow Rust API guidelines and use `rustfmt`
- **Cairo**: Follow StarkNet Cairo coding conventions
- **Documentation**: Update relevant docs when changing functionality
- **Tests**: Add tests for new features and bug fixes

## License

**MIT License**

Copyright (c) 2024 Pitchlake Team

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

## Support

- **Documentation**: Start with [Quick Start](docs/getting-started/quickstart.md)
- **Issues**: Report bugs and request features via GitHub Issues
- **Integration**: See [Integration Guide](docs/guides/integration-guide.md)
- **Architecture**: Read [Architecture Overview](docs/architecture/overview.md)
