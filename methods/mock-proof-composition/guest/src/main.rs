use core::ProofCompositionInput;
use core::ProofCompositionOutput;
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
        reserve_price_start_timestamp: data.start_timestamp, // Reserve price uses 3-month range
        reserve_price_end_timestamp: data.end_timestamp,
        reserve_price: to_fixed_packed_hex(data.reserve_price),
        twap_start_timestamp: data.start_timestamp, // TWAP uses 3-month range
        twap_end_timestamp: data.end_timestamp,
        twap_result: to_fixed_packed_hex(data.twap_result),
        max_return_start_timestamp: data.data_8_months_start_timestamp, // Max return uses 8-month range
        max_return_end_timestamp: data.data_8_months_end_timestamp,
        max_return: to_fixed_packed_hex(data.max_return),
        floating_point_tolerance: to_fixed_packed_hex(data.floating_point_tolerance),
        reserve_price_tolerance: to_fixed_packed_hex(data.reserve_price_tolerance),
        gradient_tolerance: to_fixed_packed_hex(data.gradient_tolerance),
        twap_tolerance: to_fixed_packed_hex(data.twap_tolerance),
    };

    env::commit(&output);
}
