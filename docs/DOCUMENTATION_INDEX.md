# Pitchlake Coprocessor Documentation Index

Complete documentation for the Pitchlake Coprocessor zero-knowledge proof system.

## Documentation Structure

This documentation is organized to support different user journeys:

- **New Users** → Start with Getting Started
- **Developers** → Read Architecture then Guides
- **Integrators** → Focus on Integration Guide and API Reference
- **Contributors** → Review Architecture and Contributing guidelines

## 📚 Getting Started

Essential guides to get up and running quickly.

| Document | Description | Time to Complete |
|----------|-------------|------------------|
| [Installation Guide](getting-started/installation.md) | Install dependencies and build the project | 15-20 minutes |
| [Quick Start](getting-started/quickstart.md) | Run your first proof in 5 minutes | 5 minutes |
| [Testing Guide](getting-started/testing.md) | Run tests and benchmarks | 10 minutes |

**Recommended path for new users:**
1. Installation Guide → 2. Quick Start → 3. Testing Guide

## 🏗️ Architecture

Deep dive into system design and implementation.

| Document | Description | Audience |
|----------|-------------|----------|
| [Architecture Overview](architecture/overview.md) | High-level system design and components | All developers |
| [Proof Composition](architecture/proof-composition.md) | How seven sub-proofs compose into final proof | zkVM developers |
| [Methods Overview](architecture/methods-overview.md) | Detailed breakdown of each zkVM method | Core contributors |

**Key concepts covered:**
- Proof composition architecture
- Sub-proof generation and verification
- Data flow and processing pipeline
- Fixed-point and floating-point arithmetic
- Tolerance and error bounds

## 📖 Guides

Practical guides for common tasks and workflows.

| Document | Description | Use Case |
|----------|-------------|----------|
| [Environment Setup](guides/environment-setup.md) | Configure environment variables and data files | Local development |
| [Running Methods](guides/running-methods.md) | Execute individual methods and full composition | Testing, debugging |
| [Integration Guide](guides/integration-guide.md) | Integrate coprocessor with your application | External integration |
| [On-Chain Integration](../OnChain-Integration-Summary.md) | Deploy and verify proofs on StarkNet | Blockchain deployment |

**Common workflows:**
- Setting up local development environment
- Running individual sub-proofs for testing
- Generating production proofs
- Integrating with external applications
- Deploying to StarkNet

## 📦 Methods

Detailed documentation for each zkVM method.

| Method | Input | Output | Purpose |
|--------|-------|--------|---------|
| [Hashing Felts](methods/hashing-felts.md) | Gas fee data (5760 hours) | 32-byte hash | Data integrity verification |
| [Max Return](methods/max-return.md) | 8-month data | Maximum return | Volatility measurement |
| [TWAP](methods/twap.md) | 3-month data | Time-weighted average | Fair pricing |
| [Remove Seasonality](methods/remove-seasonality.md) | 3-month data + params | Verification | Detrend time series |
| [Add TWAP 7D](methods/add-twap-7d.md) | 3-month data | 7-day TWAPs | Rolling average |
| [Calculate Pt Pt1](methods/calculate-pt-pt1.md) | Deseasonalized data | Transition matrices | Markov model |
| [Simulate Price](methods/simulate-price.md) | All params | Reserve price | Monte Carlo simulation |

**Proof flow:**
```
Data Input → [Hashing, Max Return, TWAP] (Parallel)
          → [Remove Seasonality, Add TWAP 7D, Calculate Pt/Pt1] (Sequential)
          → [Simulate Price] (Final)
          → Proof Composition
```

## 📜 Contracts

Cairo smart contract documentation for StarkNet integration.

| Contract | Purpose | Network |
|----------|---------|---------|
| [Pitchlake Verifier](contracts/pitchlake-verifier.md) | RISC Zero Groth16 proof verification | StarkNet Sepolia/Mainnet |
| [Mock Contracts](contracts/mocks.md) | Testing contracts for development | Local Katana |

**On-chain verification flow:**
1. Generate proof off-chain
2. Submit proof to verifier contract
3. Contract verifies Groth16 proof
4. Contract decodes journal
5. Results sent to vault contracts

## 📋 Reference

Technical reference documentation.

| Document | Description |
|----------|-------------|
| [ProofCompositionOutput](../ProofCompositionOutput.md) | Output structure and field descriptions |
| [ProofCompositionInput](../PROOF_COMPOSITION_GUIDE.md) | Input structure and data requirements |
| [On-Chain Integration Summary](../OnChain-Integration-Summary.md) | Gas optimization and minimal journal structure |

## 🎯 User Journeys

### I want to... generate my first proof

**Path:** Installation → Quick Start → Run mock proof
```bash
# 1. Install
curl -L https://risczero.com/install | bash && rzup install

# 2. Build
cargo build --release

# 3. Run
cd mains/mock-proof-composition
RISC0_DEV_MODE=1 cargo run --release
```

**Time:** 25 minutes total

### I want to... understand the system architecture

**Path:** Architecture Overview → Proof Composition → Methods Overview

**Key concepts:**
- Seven specialized sub-proofs
- Proof composition using RISC Zero's assumption feature
- Host vs guest computation
- Fixed-point encoding for determinism

**Time:** 30-45 minutes

### I want to... integrate with my application

**Path:** Integration Guide → Environment Setup → Running Methods

**Steps:**
1. Read integration guide
2. Set up data fetching from Ethereum
3. Call proof generation functions
4. Extract results from receipt
5. Submit to StarkNet

**Time:** 2-3 hours for full integration

### I want to... deploy to production

**Path:** Installation → Testing → Integration Guide → On-Chain Integration

**Checklist:**
- [ ] Install all dependencies
- [ ] Run full test suite
- [ ] Generate production proof
- [ ] Deploy verifier contract to StarkNet
- [ ] Test proof submission
- [ ] Set up monitoring

**Time:** 1 day for initial deployment

### I want to... contribute to the project

**Path:** Architecture → Methods Overview → Testing → Contributing

**Prerequisites:**
- Understand proof composition architecture
- Familiar with RISC Zero zkVM
- Experience with Rust and Cairo
- Read existing method implementations

**First contribution ideas:**
- Add tests for edge cases
- Improve documentation
- Optimize proof generation
- Add new tolerance configurations

## 📊 Performance Reference

| Operation | Dev Mode | Production Mode | Memory |
|-----------|----------|-----------------|--------|
| Mock Proof Composition | ~1 second | N/A | 2-4 GB |
| Individual Sub-Proof | ~30-60 seconds | ~2-3 minutes | 4-8 GB |
| Full Proof Composition | ~2 minutes | ~6-10 minutes | 8-16 GB |
| Proof Verification | ~100 ms | ~100 ms | < 1 GB |

## 🔧 Quick Reference

### Environment Variables

```bash
# Dev mode (fast, no real proofs)
export RISC0_DEV_MODE=1

# Production mode (slow, real proofs)
export RISC0_DEV_MODE=0

# Bonsai API (optional, for remote proving)
export BONSAI_API_KEY="your_key"

# Debug logging
export RUST_LOG=debug
```

### Common Commands

```bash
# Build all methods
cargo build --release --all

# Run all tests
cargo test --all

# Run mock proof
cd mains/mock-proof-composition && RISC0_DEV_MODE=1 cargo run --release

# Run full proof
cd mains/proof-composition-* && RISC0_DEV_MODE=0 cargo run --release

# Extract data file
gunzip -c data.csv.gz > data.csv
```

### Troubleshooting

| Issue | Solution | Reference |
|-------|----------|-----------|
| `cargo: command not found` | Install Rust | [Installation Guide](getting-started/installation.md#rust-toolchain) |
| `cargo risczero: command not found` | Install RISC Zero | [Installation Guide](getting-started/installation.md#risc-zero-toolchain) |
| `data.csv not found` | Extract data file | [Quick Start](getting-started/quickstart.md#data-file-setup) |
| Out of memory | Use dev mode or reduce jobs | [Testing Guide](getting-started/testing.md#debugging-tests) |
| Proof verification failed | Check input data | [Quick Start](getting-started/quickstart.md#troubleshooting) |

## 📝 Contributing to Documentation

### Documentation Standards

- **Clear and concise** - Avoid jargon, explain technical terms
- **Code examples** - Include runnable code snippets
- **Visual aids** - Use diagrams and tables
- **Cross-references** - Link to related documentation
- **Tested** - Verify all commands and code examples work

### Adding New Documentation

1. Follow the existing structure
2. Use consistent formatting
3. Add entry to this index
4. Link from relevant sections
5. Test all code examples

### Documentation Wishlist

Areas that need more documentation:

- [ ] Detailed method implementation guides
- [ ] Advanced proof composition patterns
- [ ] Performance optimization techniques
- [ ] Security considerations and best practices
- [ ] Migration guide from Bonsai to Boundless
- [ ] Comparison with alternative zkVM systems (SP1, etc.)

## 🔗 External Resources

**RISC Zero:**
- [RISC Zero Documentation](https://dev.risczero.com/)
- [zkVM Quick Start](https://dev.risczero.com/zkvm/quickstart)
- [Proof Composition Guide](https://dev.risczero.com/zkvm/composition)

**StarkNet:**
- [StarkNet Documentation](https://docs.starknet.io/)
- [Cairo Book](https://book.cairo-lang.org/)
- [Starknet Foundry](https://foundry-rs.github.io/starknet-foundry/)

**Rust:**
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

## 📮 Support

- **GitHub Issues** - Report bugs and request features
- **Documentation Issues** - Report documentation problems
- **Integration Help** - See [Integration Guide](guides/integration-guide.md)
- **Architecture Questions** - Read [Architecture Overview](architecture/overview.md)

---

**Last Updated:** 2024-10-14
**Documentation Version:** 1.0.0
**Project Version:** See `Cargo.toml`
