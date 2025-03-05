use benchmark::{
    original::{self, convert_array1_to_dvec, convert_array2_to_dmatrix},
    tests::mock::get_first_period_data,
};
use calculate_simulated_log_prices_floating::calculate_simulated_log_prices;
use calculate_simulated_log_prices_floating_core::CalculateSimulatedLogPricesInput;
use calculate_simulated_log_prices_floating_methods::CALCULATE_SIMULATED_LOG_PRICES_FLOATING_GUEST_ID;

fn main() {
    // let num_paths = 15000;
    let num_paths = 4000;
    let n_periods = 720;
    // get only first period of (timestamp avg_gas_fee)
    let data = get_first_period_data();
    // run rust code in host
    // ensure convergence in host
    let res = original::calculate_reserve_price(&data, num_paths, n_periods);

    let input = CalculateSimulatedLogPricesInput {
        period_start_timestamp: data[0].0,
        period_end_timestamp: data[data.len() - 1].0,
        season_param: convert_array1_to_dvec(res.season_param),
        de_seasonalized_detrended_simulated_prices: convert_array2_to_dmatrix(
            res.de_seasonalized_detrended_simulated_prices,
        ),
        twap_7d: res.twap_7d,
        slope: res.slope,
        intercept: res.intercept,
        log_base_fee_len: data.len(),
        num_paths,
        n_periods,
        supposed_simulated_log_prices: convert_array2_to_dmatrix(res.simulated_log_prices),
        error_tolerance_in_percentage: 10.0,
        matrix_tolerance_in_percentage: 5.0,
    };

    // let simulated_log_prices = calculate_simulated_log_prices_no_prove(
    //     input.period_start_timestamp,
    //     input.period_end_timestamp,
    //     &input.season_param,
    //     &input.de_seasonalized_detrended_simulated_prices,
    //     &input.twap_7d,
    //     input.slope,
    //     input.intercept,
    //     input.log_base_fee_len,
    //     input.num_paths,
    //     input.n_periods,
    // )
    // .unwrap();

    // let is_within_error_tolerance = error_bound_simulated_log_prices(
    //     &input.supposed_simulated_log_prices,
    //     &simulated_log_prices,
    //     input.error_tolerance_in_percentage,
    //     input.matrix_tolerance_in_percentage,
    // );

    // println!("is_within_error_tolerance: {:?}", is_within_error_tolerance);

    // println!("res: {:?}", res);

    let (receipt, res) = calculate_simulated_log_prices(input);

    // verify guest receipt

    receipt
        .verify(CALCULATE_SIMULATED_LOG_PRICES_FLOATING_GUEST_ID)
        .unwrap();
}
