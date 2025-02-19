use calculate_simulated_log_prices_floating_core::CalculateSimulatedLogPricesInput;
use benchmark::floating_point::calculate_simulated_log_prices;
use risc0_zkvm::guest::env;

fn main() {
    let data: CalculateSimulatedLogPricesInput = env::read();
    let simulated_log_prices = calculate_simulated_log_prices(
        data.period_start_timestamp,
        data.period_end_timestamp,
        &data.season_param,
        &data.de_seasonalized_detrended_simulated_prices,
        &data.twap_7d,
        data.slope,
        data.intercept,
        data.log_base_fee_len,
        data.num_paths,
        data.n_periods,
    );
}

// pub struct CalculateSimulatedLogPricesInput {
//     pub period_start_timestamp: i64,
//     pub period_end_timestamp: i64,
//     pub season_param: DVector<f64>,
//     pub de_seasonalized_detrended_simulated_prices: DMatrix<f64>,
//     pub twap_7d: Vec<f64>,
//     pub slope: f64,
//     pub intercept: f64,
//     pub log_base_fee_len: usize,
//     pub num_paths: usize,
//     pub n_periods: usize,
    // pub supposed_simulated_log_prices: DMatrix<f64>,
// }
