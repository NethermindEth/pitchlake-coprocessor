use common::{
    add_df_property, csv::close_csv_file, csv::open_reserve_price_csv_writer,
    csv::write_reserve_price_to_csv, read_data_from_file, split_dataframe_into_periods,
};
use reserve_price_original::calculate_reserve_price;

#[tokio::main]
async fn main() {
    let df = read_data_from_file("data.csv");
    let df = add_df_property(df);
    let periods = split_dataframe_into_periods(df, 3).unwrap();

    let mut wtr = open_reserve_price_csv_writer("reserve_price_reserve_price_original.csv");
    for period in periods {
        let reserve_price = calculate_reserve_price(period.clone()).await;
        if reserve_price.is_err() {
            continue;
        }
        let reserve_price = reserve_price.unwrap();

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
        println!("reserve_price: {}\n", reserve_price);
        write_reserve_price_to_csv(&mut wtr, start_timestamp, end_timestamp, reserve_price);
    }
    close_csv_file(&mut wtr);
}
