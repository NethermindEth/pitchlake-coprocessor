use reserve_price_composition_verify_simulated_price_floating_core::ReservePriceCompositionInput;
use risc0_zkvm::{guest::env, serde};

use add_twap_7d_error_bound_floating_core::AddTwap7dErrorBoundFloatingInput;
use add_twap_7d_error_bound_floating_methods::ADD_TWAP_7D_ERROR_BOUND_FLOATING_GUEST_ID;

use simulate_price_verify_position_floating_core::SimulatePriceVerifyPositionInput;
use simulate_price_verify_position_floating_methods::SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ID;

fn main() {
    let data: ReservePriceCompositionInput = env::read();

    let add_twap_7d_error_bound_input = AddTwap7dErrorBoundFloatingInput {
        data: data.data.clone(),
        twap_7d: data.twap_7d.clone().clone(),
        tolerance: data.floating_point_tolerance,
    };

    env::verify(
        ADD_TWAP_7D_ERROR_BOUND_FLOATING_GUEST_ID,
        &serde::to_vec(&add_twap_7d_error_bound_input).unwrap(),
    )
    .unwrap();

    let simulate_price_verify_position_input = SimulatePriceVerifyPositionInput {
        data: data.data.clone(),
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
    };

    env::verify(
        SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ID,
        &serde::to_vec(&(simulate_price_verify_position_input, data.reserve_price)).unwrap(),
    )
    .unwrap();

    env::commit(&data);
}
