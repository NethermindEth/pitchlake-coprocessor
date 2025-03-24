use common::{
    common::dataframe::{
        convert_to_timestamp_base_fee_int_tuple, read_data_from_file, replace_timestamp_with_date,
        split_dataframe_into_periods,
    },
    original,
    tests::mock::get_first_period_data,
};
use twap_error_bound_floating::calculate_twap;
use core::TwapErrorBoundInput;
use twap_error_bound_floating_methods::TWAP_ERROR_BOUND_FLOATING_GUEST_ID;

fn main() {
    let df = read_data_from_file("data.csv");
    let df = replace_timestamp_with_date(df).unwrap();
    let first_period = split_dataframe_into_periods(df, 3)
        .unwrap()
        .into_iter()
        .take(1)
        .next()
        .unwrap();
    // calculate using original algorithm using base_fee for each block
    let data: Vec<(i64, i64)> = convert_to_timestamp_base_fee_int_tuple(first_period);
    let twap_original = original::calculate_twap::calculate_twap(&data);

    println!("twap_original: {:?}", twap_original);

    let data = get_first_period_data();
    let input = TwapErrorBoundInput {
        avg_hourly_gas_fee: data.iter().map(|x| x.1).collect::<Vec<f64>>(),
        twap_tolerance: 1.0,
        twap_result: twap_original,
    };

    let (receipt, _res) = calculate_twap(input);

    receipt.verify(TWAP_ERROR_BOUND_FLOATING_GUEST_ID).unwrap();
}
