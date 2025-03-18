// Copyright (c) 2023 RISC Zero, Inc.
//
// Guest program for Mock Proof Composition
// This program generates fixed-point outputs for TWAP, MaxReturn, and ReservePrice calculations
// and commits them to the journal

use mock_proof_composition_core::{ProofCompositionOutput, ProofCompositionOutputFixed};
use risc0_zkvm::guest::env;
use guest_fixed_utils::{UFixedPoint123x128, StorePacking};

/// Helper function to convert a floating point value to a hex string representation
/// of a fixed-point packednumber using the UFixedPoint123x128 type
fn to_fixed_packed_hex(value: f64) -> String {
    UFixedPoint123x128::pack(UFixedPoint123x128::from(value)).to_hex_string()
}

fn main() {
    // Define the input data with floating-point values
    let output = ProofCompositionOutput {
        // Hash of 8 months of data
        data_8_months_hash: [
            176682157, 3315611904, 69122759, 3259044264, 
            1698705339, 1448440140, 3846648702, 370555961,
        ],
        // Timestamp boundaries (Unix timestamps)
        start_timestamp: 1708833600, // Example timestamp
        end_timestamp: 1716609600,   // Example timestamp
        
        // Financial parameters (floating point)
        reserve_price: 2436485959.4697967,
        floating_point_tolerance: 1e-5,
        reserve_price_tolerance: 5.0,
        twap_tolerance: 1.0,
        gradient_tolerance: 0.05,
        
        // Computation results (floating point)
        twap_result: 14346521680.565624,
        max_return: 1.5388637251441746,
    };

    let output_fixed = ProofCompositionOutputFixed {
        // Pass through non-floating point values
        data_8_months_hash: output.data_8_months_hash,
        start_timestamp: output.start_timestamp,
        end_timestamp: output.end_timestamp,
        
        // Convert floating point values to fixed-point hex representations
        reserve_price: to_fixed_packed_hex(output.reserve_price),
        floating_point_tolerance: to_fixed_packed_hex(output.floating_point_tolerance),
        reserve_price_tolerance: to_fixed_packed_hex(output.reserve_price_tolerance),
        twap_tolerance: to_fixed_packed_hex(output.twap_tolerance),
        gradient_tolerance: to_fixed_packed_hex(output.gradient_tolerance),
        twap_result: to_fixed_packed_hex(output.twap_result),
        max_return: to_fixed_packed_hex(output.max_return),
    };

    env::commit(&output_fixed);
}
