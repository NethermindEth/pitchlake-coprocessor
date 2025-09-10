use core::{
    AddTwap7dErrorBoundFloatingInput, CalculatePtPt1ErrorBoundFloatingInput, HashingFeltOutput,
    ProofCompositionInput, ProofCompositionOutput,
    RemoveSeasonalityErrorBoundFloatingInput, SimulatePriceVerifyPositionInput,
    TwapErrorBoundInput,
};
use risc0_zkvm::{guest::env, serde};

use add_twap_7d_error_bound_floating_methods::ADD_TWAP_7D_ERROR_BOUND_FLOATING_GUEST_ID;
use remove_seasonality_error_bound_floating_methods::REMOVE_SEASONALITY_ERROR_BOUND_FLOATING_GUEST_ID;
use calculate_pt_pt1_error_bound_floating_methods::CALCULATE_PT_PT1_ERROR_BOUND_FLOATING_GUEST_ID;
use simulate_price_verify_position_floating_methods::SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ID;
use twap_error_bound_floating_methods::TWAP_ERROR_BOUND_FLOATING_GUEST_ID;
use max_return_floating_methods::MAX_RETURN_FLOATING_GUEST_ID;
use hashing_felts_methods::HASHING_FELTS_GUEST_ID;

use guest_fixed_utils::{StorePacking, UFixedPoint123x128};

/// Helper function to convert a floating point value to a hex string representation
/// of a fixed-point packednumber using the UFixedPoint123x128 type
fn to_fixed_packed_hex(value: f64) -> String {
    UFixedPoint123x128::pack(UFixedPoint123x128::from(value)).to_hex_string()
}

fn main() {
    let data: ProofCompositionInput = env::read();

    env::verify(
        HASHING_FELTS_GUEST_ID,
        &serde::to_vec(&HashingFeltOutput {
            hash: data.data_8_months_hash,
            f64_inputs: data.data_8_months.clone(),
        })
        .unwrap(),
    )
    .unwrap();

    env::verify(
        MAX_RETURN_FLOATING_GUEST_ID,
        &serde::to_vec(&(data.data_8_months.clone(), data.max_return)).unwrap(),
    )
    .unwrap();

    let data_3_months =
        data.data_8_months[data.data_8_months.len().saturating_sub(2160)..].to_vec();

    let twap_error_bound_input = TwapErrorBoundInput {
        avg_hourly_gas_fee: data_3_months.clone(),
        twap_tolerance: data.twap_tolerance,
        twap_result: data.twap_result,
    };

    env::verify(
        TWAP_ERROR_BOUND_FLOATING_GUEST_ID,
        &serde::to_vec(&twap_error_bound_input).unwrap(),
    )
    .unwrap();

    let remove_seasonality_error_bound_input = RemoveSeasonalityErrorBoundFloatingInput {
        data: data_3_months.clone(),
        slope: data.slope,
        intercept: data.intercept,
        de_seasonalised_detrended_log_base_fee: data.de_seasonalised_detrended_log_base_fee.clone(),
        season_param: data.season_param.clone(),
        tolerance: data.floating_point_tolerance,
    };

    env::verify(
        REMOVE_SEASONALITY_ERROR_BOUND_FLOATING_GUEST_ID,
        &serde::to_vec(&remove_seasonality_error_bound_input).unwrap(),
    )
    .unwrap();

    let add_twap_7d_error_bound_input = AddTwap7dErrorBoundFloatingInput {
        data: data_3_months.clone(),
        twap_7d: data.twap_7d.clone().clone(),
        tolerance: data.floating_point_tolerance,
    };

    env::verify(
        ADD_TWAP_7D_ERROR_BOUND_FLOATING_GUEST_ID,
        &serde::to_vec(&add_twap_7d_error_bound_input).unwrap(),
    )
    .unwrap();

    let calculate_pt_pt1_error_bound_input = CalculatePtPt1ErrorBoundFloatingInput {
        de_seasonalised_detrended_log_base_fee: data.de_seasonalised_detrended_log_base_fee.clone(),
        pt: data.pt.clone(),
        pt_1: data.pt_1.clone(),
        tolerance: data.floating_point_tolerance,
    };

    env::verify(
        CALCULATE_PT_PT1_ERROR_BOUND_FLOATING_GUEST_ID,
        &serde::to_vec(&calculate_pt_pt1_error_bound_input).unwrap(),
    )
    .unwrap();

    let simulate_price_verify_position_input = SimulatePriceVerifyPositionInput {
        start_timestamp: data.start_timestamp,
        end_timestamp: data.end_timestamp,
        data_length: data_3_months.len(),
        positions: data.positions.clone(),
        pt: data.pt.clone(),
        pt_1: data.pt_1.clone(),
        gradient_tolerance: data.gradient_tolerance,
        de_seasonalised_detrended_log_base_fee: data.de_seasonalised_detrended_log_base_fee.clone(),
        n_periods: data.n_periods,
        num_paths: data.num_paths,
        season_param: data.season_param.clone(),
        twap_7d: data.twap_7d.clone(),
        slope: data.slope,
        intercept: data.intercept,
        reserve_price: data.reserve_price,
        tolerance: data.reserve_price_tolerance,
    };

    env::verify(
        SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ID,
        &serde::to_vec(&simulate_price_verify_position_input).unwrap(),
    )
    .unwrap();

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
