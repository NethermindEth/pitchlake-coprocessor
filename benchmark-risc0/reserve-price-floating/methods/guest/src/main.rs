use benchmark::floating_point::calculate_reserve_price;
use nalgebra::DVector;
use reserve_price_floating_core::ReservePriceFloatingInput;
use risc0_zkvm::guest::env;

fn main() {
    let data: ReservePriceFloatingInput = env::read();
    let res = calculate_reserve_price(
        data.period_start_timestamp,
        data.period_end_timestamp,
        &data.season_param,
        &data.de_seasonalized_detrended_simulated_prices,
        &data.twap_7d,
        data.slope,
        data.intercept,
        data.log_base_fee_len,
        15000,
        720,
    )
    .unwrap();

    // (ReservePriceFloatingInput, f64)
    env::commit(&(data, res));
}

// pub fn calculate_reserve_price(
//     period_start_timestamp: i64,
//     period_end_timestamp: i64,
//     season_param: &DVector<f64>,
//     de_seasonalized_detrended_simulated_prices: &DMatrix<f64>,
//     twap_7d: &[f64],
//     slope: f64,
//     intercept: f64,
//     log_base_fee_len: usize,
// ) -> Result<f64> {
