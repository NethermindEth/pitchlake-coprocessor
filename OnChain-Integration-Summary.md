# On-Chain Integration and Gas Optimization Summary

## Overview

The `ProofCompositionOutput` struct is decoded on-chain in the Starknet verifier contract and passed to vault callbacks. This document analyzes current usage and recommends optimizations for gas efficiency.

## Current On-Chain Usage

### Starknet Verifier Flow
1. **Proof Verification**: Groth16 proof is verified on-chain
2. **Journal Decoding**: Raw journal bytes are decoded into `ProofCompositionOutput` structure
3. **Vault Callback**: Selected fields are serialized and sent to vault contract
4. **Event Emission**: All fields are emitted in `PitchlakeProofVerified` event

### Current Field Distribution

**Sent to Vault Callback (9 fields):**
- `start_timestamp` (8 bytes) - Time period start
- `end_timestamp` (8 bytes) - Time period end
- `reserve_price` (32 bytes) - Primary business output
- `twap_result` (32 bytes) - Time-weighted average price
- `max_return` (32 bytes) - Maximum return calculation
- `floating_point_tolerance` (32 bytes) - Validation tolerance
- `reserve_price_tolerance` (32 bytes) - Price validation tolerance
- `twap_tolerance` (32 bytes) - TWAP validation tolerance  
- `gradient_tolerance` (32 bytes) - Gradient validation tolerance

**Event Only (1 field):**
- `data_8_months_hash` (32 bytes) - Data integrity hash

**Total Current Size:** ~280 bytes per transaction

## Why Minimize On-Chain Data

### Gas Cost Impact
- **Storage costs**: Each felt252 (~32 bytes) costs significant gas
- **Calldata costs**: Data passed between contracts incurs gas fees
- **Network congestion**: Larger transactions compete for block space

### Trust Model
- **Zero-knowledge proofs already validate correctness** - If the proof verifies, all computations were performed correctly within specified tolerances
- **Redundant validation** - Sending tolerance values on-chain duplicates validation already proven

### Business Logic Requirements
Most vault contracts only need the **computed results**, not the **validation parameters** used during computation.

## Recommended Minimal Structure

### Essential Fields Only (5 fields, ~144 bytes)
```cairo
struct MinimalJournal {
    start_timestamp: u64,      // 8 bytes - Required for time bounds
    end_timestamp: u64,        // 8 bytes - Required for time bounds  
    reserve_price: felt252,    // 32 bytes - Primary business output
    twap_result: felt252,      // 32 bytes - Key financial metric
    max_return: felt252,       // 32 bytes - Risk management metric
}
```

### Why These Fields Are Essential

1. **`start_timestamp` / `end_timestamp`**
   - Define the exact time period for financial calculations
   - Critical for contract logic and compliance
   - Cannot be derived from other data

2. **`reserve_price`**
   - Primary output that drives business decisions
   - Core value the entire computation aims to produce
   - Directly impacts vault operations

3. **`twap_result`**
   - Time-weighted average price over the period
   - Essential for trading and pricing decisions
   - Cannot be recalculated without full historical data

4. **`max_return`**
   - Maximum return achieved in the period
   - Critical for risk assessment and compliance
   - Used in vault strategy decisions

## Why Remove Other Fields

### Tolerance Fields (Can Remove)
- `floating_point_tolerance`, `reserve_price_tolerance`, `twap_tolerance`, `gradient_tolerance`
- **Purpose**: Validation parameters during ZK computation
- **Redundant**: Proof verification already confirms tolerances were met
- **Gas Impact**: 128 bytes saved (~45% reduction)

### Data Hash (Can Remove from Callback)
- `data_8_months_hash` 
- **Purpose**: Data integrity verification
- **Alternative**: Keep in events for audit trails
- **Reasoning**: ZK proof already validates correct data usage

## Gas Savings Analysis

| Configuration | Size | Fields | Gas Savings |
|---------------|------|--------|-------------|
| Current | ~280 bytes | 10 fields | Baseline |
| Minimal | ~144 bytes | 5 fields | ~48% reduction |
| Hybrid | ~176 bytes | 6 fields | ~37% reduction |

## Implementation Strategy

### Phase 1: Safe Transition
- Keep current structure but add minimal variant
- Update contracts to accept both formats
- Monitor vault contract requirements

### Phase 2: Optimization
- Switch to minimal structure for new deployments
- Migrate existing contracts during upgrades
- Maintain backward compatibility

### Phase 3: Full Adoption
- Standardize on minimal structure
- Remove deprecated fields from new proofs
- Document breaking changes

## Risk Mitigation

### Audit Trail Preservation
- Keep full data in events for off-chain analysis
- Maintain historical compatibility
- Enable forensic reconstruction if needed

### Vault Contract Compatibility
- Survey existing vault implementations
- Ensure no critical dependencies on tolerance fields
- Provide migration guidelines

## Conclusion

**Recommended Action**: Implement minimal structure with 5 essential fields, saving ~48% in gas costs while maintaining all business-critical functionality. The zero-knowledge proof system already guarantees computational correctness, making validation parameters redundant on-chain.