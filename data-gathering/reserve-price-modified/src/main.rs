use benchmark::{
    common::{
        csv::{close_csv_file, open_reserve_price_csv_writer, write_reserve_price_to_csv},
        dataframe::{
            add_df_property, convert_to_timestamp_base_fee_tuple, read_data_from_file,
            split_dataframe_into_periods,
        },
    },
    floating_point::calculate_reserve_price_full,
};

fn main() {
    let df = read_data_from_file("data.csv");
    let df = add_df_property(df);
    let periods = split_dataframe_into_periods(df, 3).unwrap();

    let mut wtr =
        open_reserve_price_csv_writer("reserve_price_reserve_price_modified_2400_iter.csv");
    for period in periods {
        let timestamp_gas_used_tuple = convert_to_timestamp_base_fee_tuple(period.clone());
        let all_inputs_to_reserve_price = calculate_reserve_price_full(&timestamp_gas_used_tuple);

        let start_timestamp = period
            .column("date")
            .unwrap()
            .datetime()
            .unwrap()
            .get(0)
            .unwrap();
        let end_timestamp = period
            .column("date")
            .unwrap()
            .datetime()
            .unwrap()
            .get(period.height() - 1)
            .unwrap();

        println!("start_timestamp: {}", start_timestamp);
        println!("end_timestamp: {}", end_timestamp);
        println!(
            "reserve_price: {}\n",
            all_inputs_to_reserve_price.reserve_price
        );
        write_reserve_price_to_csv(
            &mut wtr,
            start_timestamp,
            end_timestamp,
            all_inputs_to_reserve_price.reserve_price,
        );
    }

    close_csv_file(&mut wtr);
}
