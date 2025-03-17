use common::floating_point::{
    calculate_reserve_price, error_bound_f64, simulate_price_verify_position,
};
use risc0_zkvm::guest::env;
use simulate_price_verify_position_floating_core::SimulatePriceVerifyPositionInput;

// TODO: error bound check for reserve_price
fn main() {
    let data: SimulatePriceVerifyPositionInput = env::read();

    let (is_saddle_point, de_seasonalized_detrended_simulated_prices) =
        simulate_price_verify_position(
            &data.positions,
            &data.pt,
            &data.pt_1,
            data.gradient_tolerance,
            &data.de_seasonalised_detrended_log_base_fee,
            data.n_periods,
            data.num_paths,
        );

    assert!(is_saddle_point);

    let reserve_price = calculate_reserve_price(
        data.start_timestamp,
        data.end_timestamp,
        &data.season_param,
        &de_seasonalized_detrended_simulated_prices,
        &data.twap_7d,
        data.slope,
        data.intercept,
        data.data_length,
        data.num_paths,
        data.n_periods,
    )
    .unwrap();

    let is_within_tolerance_reserve_price =
        error_bound_f64(reserve_price, data.reserve_price, data.tolerance);
    assert!(is_within_tolerance_reserve_price);

    env::commit(&data);
}
