# ProofCompositionOutput Documentation

## Overview

`ProofCompositionOutput` is a core data structure defined in `methods/core/src/lib.rs:62-73` that represents the output of zero-knowledge proof composition computations in the Pitchlake coprocessor system.

## Structure Definition

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProofCompositionOutput {
    pub data_8_months_hash: [u32; 8],
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub reserve_price: String,
    pub floating_point_tolerance: String,
    pub reserve_price_tolerance: String,
    pub twap_tolerance: String,
    pub gradient_tolerance: String,
    pub twap_result: String,
    pub max_return: String,
}
```

## Field Descriptions

- **`data_8_months_hash`**: A hash of the 8-month data input as an array of 8 u32 values
- **`start_timestamp`**: Unix timestamp indicating the start of the data period
- **`end_timestamp`**: Unix timestamp indicating the end of the data period
- **`reserve_price`**: Hex-encoded fixed-point representation of the reserve price value
- **`floating_point_tolerance`**: Hex-encoded tolerance for floating point comparisons
- **`reserve_price_tolerance`**: Hex-encoded tolerance for reserve price calculations
- **`twap_tolerance`**: Hex-encoded tolerance for TWAP (Time-Weighted Average Price) calculations
- **`gradient_tolerance`**: Hex-encoded tolerance for gradient calculations
- **`twap_result`**: Hex-encoded result of TWAP calculations
- **`max_return`**: Hex-encoded maximum return value

## Usage Patterns

### 1. RISC-V Zero-Knowledge Guest Programs

The struct is primarily used as the output format for RISC-V zero-knowledge guest programs that commit their results to a journal:

#### Mock Proof Composition Guest
**File**: `methods/mock-proof-composition/guest/src/main.rs:17-28`

```rust
let output = ProofCompositionOutput {
    data_8_months_hash: data.data_8_months_hash,
    start_timestamp: data.start_timestamp,
    end_timestamp: data.end_timestamp,
    reserve_price: to_fixed_packed_hex(data.reserve_price),
    floating_point_tolerance: to_fixed_packed_hex(data.floating_point_tolerance),
    reserve_price_tolerance: to_fixed_packed_hex(data.reserve_price_tolerance),
    gradient_tolerance: to_fixed_packed_hex(data.gradient_tolerance),
    twap_tolerance: to_fixed_packed_hex(data.twap_tolerance),
    twap_result: to_fixed_packed_hex(data.twap_result),
    max_return: to_fixed_packed_hex(data.max_return),
};

env::commit(&output);
```

#### Full Proof Composition Guest
**File**: `methods/proof-composition-twap-maxreturn-reserveprice-floating-hashing-methods/guest/src/main.rs`

Similar pattern where the output is constructed and committed to the RISC-V journal after performing various verification steps.

### 2. Host Program Integration

#### Mock Proof Composition Host
**File**: `mains/mock-proof-composition/src/main.rs:66-67`

The host program decodes the journal output from the guest:

```rust
let decoded_journal = receipt.journal.decode::<ProofCompositionOutput>();
println!("DECODED JOURNAL: {:?}", decoded_journal);
```

This decoded output is then used to generate Groth16 proofs for Ethereum integration.

## Tolerance Usage and Purpose

The tolerance fields serve as **error bounds for floating-point calculations** in the zero-knowledge proof system:

### 1. **floating_point_tolerance**
General tolerance for floating-point comparisons to handle precision errors inherent in floating-point arithmetic.

### 2. **gradient_tolerance** 
Used in `CalculatePtPt1ErrorBoundFloatingInput` for validating that calculated gradients (pt and pt_1 vectors) are within acceptable bounds of expected values.

### 3. **reserve_price_tolerance**
Validates that calculated reserve prices fall within an acceptable range of the expected reserve price value.

### 4. **twap_tolerance** 
Used in `TwapErrorBoundInput` to verify that TWAP (Time-Weighted Average Price) calculations are within acceptable bounds of the expected result.

### Why Tolerances Are Critical

In zero-knowledge proofs, **deterministic computation is essential**. However, floating-point arithmetic can produce slightly different results across different systems due to:
- Rounding errors
- Different floating-point implementations  
- Order of operations

The tolerances allow the system to:
1. **Accept slight variations** in calculations while maintaining correctness
2. **Prevent proof failures** due to minor precision differences
3. **Ensure reproducibility** across different computing environments
4. **Validate economic parameters** (prices, returns) are within reasonable bounds

### Implementation Pattern

The guest programs use assertions like:
```rust
assert!(is_within_tolerance_reserve_price);
assert!(is_within_tolerance_pt_1);
```

These assertions verify that calculated values fall within the specified tolerance ranges before committing results to the proof journal, ensuring the zero-knowledge proof only succeeds when calculations are sufficiently accurate.

## Fixed-Point Encoding

The numeric fields (all String types) are encoded using a fixed-point representation through the `UFixedPoint123x128` type:

```rust
fn to_fixed_packed_hex(value: f64) -> String {
    UFixedPoint123x128::pack(UFixedPoint123x128::from(value)).to_hex_string()
}
```

This ensures deterministic representation of floating-point values in the zero-knowledge proof system.

## Integration Flow

1. **Input Processing**: `ProofCompositionInput` contains raw f64 values
2. **Guest Computation**: RISC-V guest programs perform calculations and validations
3. **Output Generation**: Results are converted to fixed-point hex strings and packaged in `ProofCompositionOutput`
4. **Journal Commitment**: The output is committed to the RISC-V journal using `env::commit(&output)`
5. **Host Decoding**: Host programs decode the journal to retrieve the structured output
6. **Proof Generation**: The output is used to generate Groth16 proofs for blockchain verification

## On-Chain Integration and Gas Optimization

### Current Starknet Usage

The `ProofCompositionOutput` is decoded on-chain in the Starknet verifier contract (`/home/ametel/source/fossil-monorepo/starknet-contracts/pitchlake-verifier/src/pitchlake_verifier.cairo`):

#### Fields Sent to Vault Callback:
- `start_timestamp`, `end_timestamp` - Time bounds for financial contracts
- `reserve_price`, `twap_result`, `max_return` - Primary business outputs
- All tolerance fields: `floating_point_tolerance`, `reserve_price_tolerance`, `twap_tolerance`, `gradient_tolerance`

#### Fields Only in Events:
- `data_8_months_hash` - Used for audit trails but not sent to vault callback

### Gas Optimization Recommendations

#### Essential Fields (Must Keep):
1. **`start_timestamp`** & **`end_timestamp`** - Critical time bounds for financial contracts
2. **`reserve_price`** - Primary business output the vault requires
3. **`twap_result`** - Key financial metric for trading decisions  
4. **`max_return`** - Essential for risk management calculations

#### Consider Removing for Gas Optimization:

1. **`data_8_months_hash`** - Only used in events, not business logic. The zero-knowledge proof already validates correct data usage.

2. **Tolerance fields** - These were validation parameters during computation:
   - `floating_point_tolerance`
   - `reserve_price_tolerance` 
   - `twap_tolerance`
   - `gradient_tolerance`
   
   Since the proof succeeded, tolerances were satisfied. May not be needed on-chain unless the vault performs additional validation.

#### Minimal Viable Structure (Significant Gas Savings):
```cairo
struct MinimalJournal {
    start_timestamp: u64,
    end_timestamp: u64, 
    reserve_price: felt252,
    twap_result: felt252,
    max_return: felt252,
}
```

#### Hybrid Approach:
Keep business-critical fields + `data_8_months_hash` for audit trails, but remove tolerance fields unless the vault specifically needs them for additional validation.

## Related Structures

- **`ProofCompositionInput`**: The corresponding input structure containing raw calculation parameters
- **`UFixedPoint123x128`**: Fixed-point number representation used for deterministic encoding

## Files Using This Structure

- `methods/core/src/lib.rs` (definition)
- `methods/mock-proof-composition/guest/src/main.rs` (construction and commitment)
- `methods/proof-composition-twap-maxreturn-reserveprice-floating-hashing-methods/guest/src/main.rs` (construction and commitment)
- `mains/mock-proof-composition/src/main.rs` (decoding and proof generation)
- `README.md` (documentation reference)