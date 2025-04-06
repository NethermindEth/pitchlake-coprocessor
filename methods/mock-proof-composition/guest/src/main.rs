use core::ProofCompositionOutput;
use core::ProofCompositionInput;
use risc0_zkvm::guest::env;

use guest_fixed_utils::{StorePacking, UFixedPoint123x128};

/// Helper function to convert a floating point value to a hex string representation
/// of a fixed-point packednumber using the UFixedPoint123x128 type
fn to_fixed_packed_hex(value: f64) -> String {
    UFixedPoint123x128::pack(UFixedPoint123x128::from(value)).to_hex_string()
}

fn main() {
    // Create mock data for ProofCompositionInput
    let data: ProofCompositionInput = env::read();

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
}
