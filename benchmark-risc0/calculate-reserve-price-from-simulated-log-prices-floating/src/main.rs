use benchmark::{
    original::{self, convert_array2_to_dmatrix},
    tests::mock::get_first_period_data,
};
use calculate_reserve_price_from_simulated_log_prices_floating::calculate_reserve_price_from_simulated_log_prices;
use calculate_reserve_price_from_simulated_log_prices_floating_core::CalculateReservePriceFromSimulatedLogPricesInput;
use calculate_reserve_price_from_simulated_log_prices_floating_methods::CALCULATE_RESERVE_PRICE_FROM_SIMULATED_LOG_PRICES_FLOATING_GUEST_ID;
fn main() {
    // let num_paths = 15000;
    let num_paths = 4000;
    let n_periods = 720;
    // get only first period of (timestamp avg_gas_fee)
    let data = get_first_period_data();
    let res = original::calculate_reserve_price(&data, num_paths, n_periods);
    println!("original reserve_price: {:?}", res.reserve_price);

    let input = CalculateReservePriceFromSimulatedLogPricesInput {
        simulated_log_prices: convert_array2_to_dmatrix(res.simulated_log_prices),
        twap_7d: res.twap_7d,
        n_periods,
    };

    let (receipt, res) = calculate_reserve_price_from_simulated_log_prices(input);
    println!("res.reserve_price: {:?}", res.1);

    // verify guest receipt

    receipt
        .verify(CALCULATE_RESERVE_PRICE_FROM_SIMULATED_LOG_PRICES_FLOATING_GUEST_ID)
        .unwrap();
}