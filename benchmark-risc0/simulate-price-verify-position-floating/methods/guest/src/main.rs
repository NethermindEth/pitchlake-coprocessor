use benchmark::floating_point::simulate_price_verify_position;
use risc0_zkvm::guest::env;
use simulate_price_verify_position_floating_core::SimulatePriceVerifyPositionInput;

// TODO: error bound check for simulated_prices
fn main() {
    let data: SimulatePriceVerifyPositionInput = env::read();

    let num_paths = 4000;
    let n_periods = 720;

    let (is_saddle_point, simulated_prices) = simulate_price_verify_position(
        &data.positions,
        &data.pt,
        &data.pt_1,
        data.gradient_tolerance,
        &data.de_seasonalised_detrended_log_base_fee,
        n_periods,
        num_paths,
    );

    assert!(is_saddle_point);

    // (SimulatePriceVerifyPositionInput, DMatrix<f64>)
    env::commit(&(data, simulated_prices));
}
