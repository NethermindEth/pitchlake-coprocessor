use benchmark::original::calculate_reserve_price;
use common::{
    add_df_property, convert_to_timestamp_base_fee_tuple, read_data_from_file,
    split_dataframe_into_periods,
};

fn main() {
    let df = read_data_from_file("data.csv");
    let df = add_df_property(df);
    let periods = split_dataframe_into_periods(df, 3).unwrap();

    let timestamp_gas_used_tuple: Vec<(i64, f64)> = convert_to_timestamp_base_fee_tuple(periods[0].clone());
    let res = calculate_reserve_price(&timestamp_gas_used_tuple);

    // println!("res: {:?}", res);
}
