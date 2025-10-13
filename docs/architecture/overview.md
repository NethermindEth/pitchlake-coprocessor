# Architecture Overview

This document provides a high-level overview of the Pitchlake Coprocessor architecture, design principles, and key components.

## Table of Contents
- [System Overview](#system-overview)
- [Design Principles](#design-principles)
- [Component Architecture](#component-architecture)
- [Data Flow](#data-flow)
- [Technology Stack](#technology-stack)
- [Future Considerations](#future-considerations)

## System Overview

The Pitchlake Coprocessor is a **zero-knowledge proof system** for verifiable financial computation. It generates cryptographic proofs that gas fee option pricing calculations (reserve price, TWAP, max return) were performed correctly using RISC Zero's zkVM.

### What It Does

**Input:** 5760 hours (240 days) of Ethereum L1 gas fee data

**Processing:** Seven specialized zkVM computations:
1. Hash data for integrity verification
2. Calculate maximum return (volatility)
3. Calculate time-weighted average price (TWAP)
4. Remove seasonality from time series
5. Calculate 7-day rolling TWAP
6. Calculate Markov transition matrices
7. Simulate prices and verify positions

**Output:** `ProofCompositionOutput` containing:
- Reserve price (option strike price)
- TWAP (fair pricing metric)
- Max return (risk metric)
- Data hash (integrity verification)
- Timestamps and tolerances

**Verification:** Proof can be verified on-chain via StarkNet smart contracts

### Why Zero-Knowledge Proofs?

Traditional systems require trusting the computation provider. With zero-knowledge proofs:

- **Verifiable Correctness** - Mathematical guarantee calculations were done correctly
- **No Trust Required** - Anyone can verify the proof without re-executing
- **Privacy Preserving** - Input data can remain private (though not used in this system)
- **Efficient Verification** - On-chain verification is fast and cheap

## Design Principles

### 1. Proof Composition

Instead of one monolithic proof, the system uses **seven specialized sub-proofs** composed into a single proof:

```
┌─────────────────────────────────────────────────┐
│              SUB-PROOFS (Parallel)              │
├─────────────────────────────────────────────────┤
│  [1] Hash           → data_8_months_hash        │
│  [2] Max Return     → max_return                │
│  [3] TWAP           → twap_result               │
│  [4] Seasonality    → Verified ✓                │
│  [5] TWAP 7D        → Verified ✓                │
│  [6] Pt/Pt1         → Verified ✓                │
│  [7] Simulate       → reserve_price ✓           │
└─────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────┐
│          PROOF COMPOSITION (Final)              │
│  Combines all sub-proofs into single proof      │
└─────────────────────────────────────────────────┘
```

**Benefits:**
- **Modularity** - Each sub-proof can be tested independently
- **Parallelization** - Sub-proofs can be generated concurrently
- **Maintainability** - Update individual methods without affecting others
- **Reusability** - Sub-proofs can be reused in different compositions

### 2. Host/Guest Separation

**Host Program** (Rust, runs on regular CPU):
- Loads data
- Performs expensive calculations
- Calls guest programs
- Manages proof composition

**Guest Program** (Rust, runs in zkVM):
- Receives pre-computed results from host
- Verifies calculations are correct
- Commits results to journal

**Why this split?**
- Guest execution is expensive (zkVM overhead)
- Host can use optimized libraries (BLAS, LAPACK)
- Only verification needs to be proven, not computation

### 3. Fixed-Point Determinism

Floating-point arithmetic is non-deterministic across platforms. The system uses:

**Host:** `f64` floating-point for calculations (fast, uses hardware FPU)

**Guest:** Fixed-point `UFixedPoint123x128` for journal output (deterministic)

**Conversion:**
```rust
// Host calculates
let reserve_price: f64 = calculate_reserve_price(&data);

// Convert to fixed-point for journal
let reserve_price_hex: String = UFixedPoint123x128::pack(
    UFixedPoint123x128::from(reserve_price)
).to_hex_string();

// Commit to journal
output.reserve_price = reserve_price_hex;
```

**Benefits:**
- Fast host computation with optimized libraries
- Deterministic journal output for verification
- Cross-platform reproducibility

### 4. Error Bounds and Tolerances

Since financial calculations involve floating-point arithmetic, the system uses **error bounds**:

```rust
pub struct ProofCompositionInput {
    // ... data fields ...

    // Tolerances
    pub floating_point_tolerance: f64,   // 0.00001 (0.00001%)
    pub gradient_tolerance: f64,         // 0.05 (5%)
    pub reserve_price_tolerance: f64,    // 5.0 (5%)
    pub twap_tolerance: f64,             // 1.0 (1%)
}
```

Guest programs verify that calculated values fall within tolerance:

```rust
// In guest program
let calculated = calculate_value(&data);
let is_within_tolerance = (calculated - expected).abs() <= tolerance;
assert!(is_within_tolerance, "Value outside tolerance");
```

**Why tolerances?**
- Handle floating-point rounding errors
- Account for different calculation orders
- Ensure reproducibility across platforms

### 5. Modular Method Design

Each method follows a consistent structure:

```
method-name-methods/
├── Cargo.toml              # Method-specific dependencies
├── build.rs                # Guest program compilation
├── src/lib.rs              # Host interface
└── guest/
    ├── Cargo.toml          # Guest dependencies
    └── src/main.rs         # Guest program (runs in zkVM)
```

**Host interface (`src/lib.rs`):**
```rust
pub fn method_name(input: Input) -> (Receipt, Output) {
    let env = ExecutorEnv::builder()
        .write(&input)?
        .build()?;

    let prover = default_prover();
    let prove_info = prover.prove(env, GUEST_ELF)?;

    (prove_info.receipt, output)
}
```

**Guest program (`guest/src/main.rs`):**
```rust
fn main() {
    let input: Input = env::read();

    // Perform verification
    let result = verify_calculation(&input);
    assert!(result.is_valid());

    // Commit output
    env::commit(&output);
}
```

## Component Architecture

### Core Components

```
┌──────────────────────────────────────────────────────┐
│                   COMMON LIBRARY                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │ Fixed Point │  │Floating Point│  │  Financial  │  │
│  │  Arithmetic │  │  Algorithms  │  │   Models    │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  │
└──────────────────────────────────────────────────────┘
                      │
                      ▼
┌──────────────────────────────────────────────────────┐
│                 ZKVM METHODS (Guest Programs)        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐           │
│  │ Hashing  │  │ Max Ret  │  │  TWAP    │  ...      │
│  └──────────┘  └──────────┘  └──────────┘           │
└──────────────────────────────────────────────────────┘
                      │
                      ▼
┌──────────────────────────────────────────────────────┐
│              HOST PROGRAMS (Mains)                   │
│  ┌──────────────┐  ┌──────────────┐                 │
│  │ Individual   │  │    Proof     │                  │
│  │   Methods    │  │ Composition  │                  │
│  └──────────────┘  └──────────────┘                 │
└──────────────────────────────────────────────────────┘
                      │
                      ▼
┌──────────────────────────────────────────────────────┐
│              STARKNET CONTRACTS                      │
│  ┌────────────────────────────────────┐              │
│  │      Pitchlake Verifier            │              │
│  │  (Groth16 Proof Verification)      │              │
│  └────────────────────────────────────┘              │
└──────────────────────────────────────────────────────┘
```

### Common Library (`common/`)

**Purpose:** Shared functionality across all methods

**Contents:**
- `fixed_point/` - Fixed-point arithmetic types
- `floating_point/` - Floating-point calculations (TWAP, etc.)
- `original/` - Financial models (reserve price, Monte Carlo)
- `tests/mock/` - Test data generators

**Key type:**
```rust
pub struct UFixedPoint123x128 {
    value: [u32; 4],  // 123 bits integer, 128 bits fractional
}
```

### zkVM Methods (`methods/`)

**Structure:** Each method has a host crate and guest program

**Key methods:**

| Method | Input Size | Output | Execution Time (Dev) |
|--------|------------|--------|---------------------|
| `hashing-felts` | 5760 felts | 32-byte hash | ~10 seconds |
| `max-return-floating` | 5760 prices | Max return | ~15 seconds |
| `twap-error-bound-floating` | 2160 prices | TWAP | ~20 seconds |
| `remove-seasonality-error-bound-floating` | 2160 + params | Verified | ~30 seconds |
| `add-twap-7d-error-bound-floating` | 2160 prices | Verified | ~25 seconds |
| `calculate-pt-pt1-error-bound-floating` | Matrices | Verified | ~20 seconds |
| `simulate-price-verify-position-floating` | All params | Reserve price | ~40 seconds |

### Host Programs (`mains/`)

**Purpose:** Execute methods and manage proof generation

**Types:**

1. **Individual Method Mains** - Test single methods
   - `mains/twap-error-bound-floating/`
   - `mains/max-return-floating/`
   - etc.

2. **Proof Composition Main** - Full proof generation
   - `mains/proof-composition-twap-maxreturn-reserveprice-floating-hashing/`

3. **Mock Composition** - Fast testing
   - `mains/mock-proof-composition/`

### StarkNet Contracts (`pitchlake_verifier/`)

**Purpose:** On-chain proof verification

**Main contract:** `pitchlake_verifier.cairo`

**Key functions:**
```cairo
// Verify RISC Zero Groth16 proof
fn verify_proof(
    proof: Span<felt252>,
    public_inputs: Span<felt252>
) -> bool;

// Decode journal to ProofCompositionOutput
fn decode_journal(
    journal: Span<felt252>
) -> ProofCompositionOutput;
```

## Data Flow

### End-to-End Flow

```
1. DATA PREPARATION
   │
   ├─ Fetch 5760 hours of gas fee data from Ethereum
   ├─ Validate data completeness
   └─ Format as Vec<f64>
   │
   ▼
2. HOST COMPUTATION
   │
   ├─ Calculate reserve price (Monte Carlo simulation)
   ├─ Extract time series parameters
   ├─ Compute Markov transition matrices
   └─ Prepare inputs for guest programs
   │
   ▼
3. SUB-PROOF GENERATION (Parallel)
   │
   ├─ Hash data → receipt_1
   ├─ Verify max return → receipt_2
   ├─ Verify TWAP → receipt_3
   ├─ Verify seasonality removal → receipt_4
   ├─ Verify 7D TWAP → receipt_5
   ├─ Verify Pt/Pt1 matrices → receipt_6
   └─ Verify simulation → receipt_7
   │
   ▼
4. PROOF COMPOSITION
   │
   ├─ Combine all 7 receipts as assumptions
   ├─ Generate final proof
   └─ Verify composition proof
   │
   ▼
5. OUTPUT EXTRACTION
   │
   ├─ Decode journal → ProofCompositionOutput
   ├─ Extract reserve_price, twap_result, max_return
   └─ Convert hex strings to numeric values
   │
   ▼
6. ON-CHAIN SUBMISSION (Optional)
   │
   ├─ Submit proof to StarkNet verifier contract
   ├─ Contract verifies Groth16 proof
   ├─ Contract decodes journal
   └─ Results forwarded to vault contracts
```

### Proof Composition Flow

RISC Zero's composition feature allows using previous proofs as **assumptions**:

```rust
// Generate sub-proofs
let (hash_receipt, _) = hash_felts(hash_input);
let (twap_receipt, _) = calculate_twap(twap_input);
let (max_ret_receipt, _) = max_return(max_ret_input);
// ... more sub-proofs ...

// Compose proofs
let env = ExecutorEnv::builder()
    .add_assumption(hash_receipt)       // Assume hash proof is valid
    .add_assumption(twap_receipt)       // Assume TWAP proof is valid
    .add_assumption(max_ret_receipt)    // Assume max return proof is valid
    // ... add more assumptions ...
    .write(&composition_input)?
    .build()?;

// Generate composed proof
let prove_info = default_prover().prove(env, COMPOSITION_GUEST_ELF)?;
let receipt = prove_info.receipt;

// Final proof verifies:
// 1. All sub-proofs are valid
// 2. Composition logic is correct
// 3. Output matches expected format
```

## Technology Stack

### Zero-Knowledge Proofs
- **RISC Zero zkVM 2.3.2** - Zero-knowledge virtual machine
- **Groth16** - Proof verification scheme for StarkNet

### Programming Languages
- **Rust (stable)** - Host programs, common library, guest programs
- **Cairo 2.x** - StarkNet smart contracts

### Scientific Computing
- **nalgebra** - Linear algebra operations
- **statrs** - Statistical distributions
- **polars** (optional) - Data frame operations
- **ndarray** (optional) - N-dimensional arrays
- **OpenBLAS** (optional) - Optimized BLAS implementation

### Blockchain
- **StarkNet** - Layer 2 for proof verification
- **Scarb** - Cairo package manager
- **Starknet Foundry** - Cairo testing framework

## Future Considerations

### 1. Bonsai Prover Deprecation

RISC Zero has announced plans to deprecate Bonsai in favor of **Boundless**, a decentralized proving marketplace.

**Migration considerations:**
- Update proof submission workflows
- Handle potential latency changes
- Review cost structures
- Test decentralized prover compatibility

**Timeline:** Monitor RISC Zero announcements

### 2. Alternative zkVM Systems

For large datasets and complex computations, consider:

**SP1 (Succinct):**
- Improved performance for data-heavy workloads
- Lower proof generation latency
- Growing ecosystem support

**Trade-offs:**
- Migration effort required
- Different proof format
- Contract updates needed

**Evaluation criteria:**
- Proof generation time
- Proof size
- Verification cost
- Developer tooling

### 3. Gas Optimization

Current `ProofCompositionOutput` includes tolerance fields that may be unnecessary on-chain:

**Current:** ~280 bytes (10 fields)
**Optimized:** ~144 bytes (5 fields) - 48% savings

See [On-Chain Integration Summary](../../OnChain-Integration-Summary.md) for details.

### 4. Parallel Proof Generation

Currently sub-proofs are generated sequentially. With proper coordination:

```rust
use rayon::prelude::*;

// Generate sub-proofs in parallel
let receipts: Vec<Receipt> = vec![
    || hash_felts(hash_input),
    || max_return(max_ret_input),
    || calculate_twap(twap_input),
].par_iter()
 .map(|f| f().0)  // Extract receipt
 .collect();
```

**Benefits:**
- 3-4x speedup with 7 parallel proofs
- Better CPU utilization
- Reduced total execution time

## Next Steps

- [Proof Composition](proof-composition.md) - Detailed proof composition architecture
- [Methods Overview](methods-overview.md) - Deep dive into each method
- [Integration Guide](../guides/integration-guide.md) - Integrate with your application
