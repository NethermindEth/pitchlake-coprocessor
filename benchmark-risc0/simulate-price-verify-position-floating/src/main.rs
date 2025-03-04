use benchmark::{
    original::{calculate_reserve_price, convert_array1_to_dvec},
    tests::mock::{get_first_period_data, get_max_return_input_data},
};
use simulate_price_verify_position_floating::simulate_price_verify_position;
use simulate_price_verify_position_floating_core::SimulatePriceVerifyPositionInput;
use simulate_price_verify_position_floating_methods::SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ID;

fn main() {
    // get only first period of (timestamp avg_gas_fee)
    // let data = get_first_period_data();

    let data_8_months = get_max_return_input_data();
    let data = data_8_months[data_8_months.len().saturating_sub(2160)..].to_vec();
    // run rust code in host
    // ensure convergence in host
    let res = calculate_reserve_price(&data, 15000, 720);
    // println!("res: {:?}", res);
    // create input for guest
    println!("original reserve price: {:?}", res.reserve_price);

    let input = SimulatePriceVerifyPositionInput {
        data: data.clone(),
        positions: res.positions,
        pt: convert_array1_to_dvec(res.pt),
        pt_1: convert_array1_to_dvec(res.pt_1),
        gradient_tolerance: 5e-3, // a too small tolerance will result in the verification to fail, this is because of how floating point operation always introduce some sort of inaccuracies
        de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
            res.de_seasonalised_detrended_log_base_fee,
        ),
        n_periods: 720,
        num_paths: 4000,
        season_param: convert_array1_to_dvec(res.season_param),
        twap_7d: res.twap_7d,
        slope: res.slope,
        intercept: res.intercept,
        reserve_price: res.reserve_price,
        tolerance: 5.0, // 5%
    };

    let (receipt, simulate_price_res) = simulate_price_verify_position(input);

    receipt
        .verify(SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ID)
        .unwrap();
}
