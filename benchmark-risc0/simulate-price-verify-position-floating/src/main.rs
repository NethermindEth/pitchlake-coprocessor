use benchmark::{
    original::{calculate_reserve_price, convert_array1_to_dvec},
    tests::mock::get_first_period_data,
};
use simulate_price_verify_position_floating::simulate_price_verify_position;
use simulate_price_verify_position_floating_core::SimulatePriceVerifyPositionInput;
use simulate_price_verify_position_floating_methods::SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ID;

fn main() {
    // get only first period of (timestamp avg_gas_fee)
    let data = get_first_period_data();
    // run rust code in host
    // ensure convergence in host
    let res = calculate_reserve_price(&data, 4000, 720);
    println!("res: {:?}", res);
    // create input for guest

    let input = SimulatePriceVerifyPositionInput {
        positions: res.positions,
        gradient_tolerance: 5e-3, // a too small tolerance will result in the verification to fail, this is because of how floating point operation always introduce some sort of inaccuracies
        de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
            res.de_seasonalised_detrended_log_base_fee,
        ),
        pt: convert_array1_to_dvec(res.pt),
        pt_1: convert_array1_to_dvec(res.pt_1),
        n_periods: 720,
        num_paths: 15000,
    };

    let (receipt, res) = simulate_price_verify_position(input);

    // verify guest receipt

    receipt
        .verify(SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ID)
        .unwrap();
}
