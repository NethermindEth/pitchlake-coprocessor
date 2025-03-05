use benchmark::tests::mock::get_first_period_data;
use benchmark::{floating_point::calculate_remove_seasonality, hex_string_to_f64};
use eyre::Result;
use simulate_price_floating::simulate_price;
use simulate_price_floating_methods::SIMULATE_PRICE_FLOATING_GUEST_ID;

fn main() {
    let reserve_price_inputs = get_first_period_data();

    // calculate this in host
    let (_slope, _intercept, de_seasonalised_detrended_log_base_fee, _season_param) =
        calculate_remove_seasonality(&reserve_price_inputs.iter().map(|x| x.1).collect()).unwrap();

    // pass calculated result in host to guest
    let (receipt, res) = simulate_price(&de_seasonalised_detrended_log_base_fee);
    println!("res: {:?}", res);

    receipt.verify(SIMULATE_PRICE_FLOATING_GUEST_ID).unwrap();
}
