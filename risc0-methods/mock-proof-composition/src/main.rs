// Copyright (c) 2023 RISC Zero, Inc.
//
// Mock proof composition for TWAP, MaxReturn, ReservePrice with floating point calculations
// This program demonstrates generating and verifying proofs with fixed-point number outputs

// External imports
use risc0_zkvm::{default_prover, ExecutorEnv};

// Project imports
use guest_fixed_utils::{Felt, StorePacking, UFixedPoint123x128};
use mock_proof_composition_core::ProofCompositionOutputFixed;
use mock_proof_composition_methods::MOCK_PROOF_COMPOSITION_GUEST_ELF;

/// Formats and prints a fixed-point value with a label
fn print_fixed_value(label: &str, hex_value: &str) {
    let felt = Felt::from_hex_string(hex_value).unwrap();
    let unpacked = UFixedPoint123x128::unpack(felt);
    println!("{}: {:?}", label, unpacked);
}

fn main() {
    // Create a default executor environment
    let env = ExecutorEnv::builder().build().unwrap();

    // Generate the proof
    println!("Generating proof...");
    let prove_info = default_prover()
        .prove(env, MOCK_PROOF_COMPOSITION_GUEST_ELF)
        .unwrap();

    let receipt = prove_info.receipt;

    // Decode the journal data
    let decoded = receipt
        .journal
        .decode::<ProofCompositionOutputFixed>()
        .unwrap();

    // Print journal summary
    println!("\n=== Journal Summary ===");
    println!("Raw journal data: {:?}", decoded);

    // Print individual fixed-point values in a readable format
    println!("\n=== Fixed-Point Values ===");
    print_fixed_value("Reserve Price", &decoded.reserve_price);
    print_fixed_value(
        "Floating Point Tolerance",
        &decoded.floating_point_tolerance,
    );
    print_fixed_value("Reserve Price Tolerance", &decoded.reserve_price_tolerance);
    print_fixed_value("TWAP Tolerance", &decoded.twap_tolerance);
    print_fixed_value("Gradient Tolerance", &decoded.gradient_tolerance);
    print_fixed_value("TWAP Result", &decoded.twap_result);
    print_fixed_value("Max Return", &decoded.max_return);

    // Print raw journal bytes (may be useful for debugging)
    println!("\n=== Raw Journal Bytes ===");
    println!("{:?}", receipt.journal.bytes);
}
