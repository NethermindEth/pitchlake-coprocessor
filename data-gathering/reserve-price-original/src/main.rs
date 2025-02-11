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
    let max_iterations = 10000;

    let mut wtr = open_reserve_price_csv_writer("reserve_price_reserve_price_original.csv");
    for period in periods {
        let reserve_price = calculate_reserve_price(period.clone(), max_iterations).await;
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

// Solution { position: [-0.017633342845010558, 0.8891236203630338, 0.04937254907891192, 0.01382943862840793, 0.15257919054410768, 0.3365041893564977], value: -189.01247078357096 } 2400 iter
// Solution { position: [-0.01762796744188326, 0.8891310386302813, 0.04932426869529885, 0.013824002748472133, 0.1525309538232292, 0.33666040650388845], value: -189.01249239179717 } 3500 iter
// Solution { position: [-0.01762748615163198, 0.8891317028744364, 0.04931994818663203, 0.01382354520544229, 0.1525266305724204, 0.33667441434122203], value: -189.0124925657238 } 5000 iter
// Solution { position: [-0.01762748615163198, 0.8891317028744364, 0.04931994818663203, 0.01382354520544229, 0.1525266305724204, 0.33667441434122203], value: -189.0124925657238 } 5500 iter
// Solution { position: [-0.01762748615163198, 0.8891317028744364, 0.04931994818663203, 0.01382354520544229, 0.1525266305724204, 0.33667441434122203], value: -189.0124925657238 } 8000 iter
// Solution { position: [-0.01762748615163198, 0.8891317028744364, 0.04931994818663203, 0.01382354520544229, 0.1525266305724204, 0.33667441434122203], value: -189.0124925657238 } 10,000 iter
