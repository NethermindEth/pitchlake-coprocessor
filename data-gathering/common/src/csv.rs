use std::fs::File;

use csv::Writer;

pub fn open_csv_writer(file_name: &str) -> Writer<File> {
    let wtr = Writer::from_path(file_name).unwrap();
    wtr
}

pub fn write_to_csv(wtr: &mut Writer<File>, data: &[String]) {
    wtr.write_record(data).unwrap();
}

pub fn open_reserve_price_csv_writer(file_name: &str) -> Writer<File> {
    let mut wtr = Writer::from_path(file_name).unwrap();
    write_to_csv(
        &mut wtr,
        &[
            "start_timestamp".to_owned(),
            "end_timestamp".to_owned(),
            "reserve_price".to_owned(),
        ],
    );
    wtr
}

pub fn write_reserve_price_to_csv(
    wtr: &mut Writer<File>,
    start_timestamp: i64,
    end_timestamp: i64,
    reserve_price: f64,
) {
    write_to_csv(
        wtr,
        &[
            start_timestamp.to_string(),
            end_timestamp.to_string(),
            reserve_price.to_string(),
        ],
    );
}

pub fn close_csv_file(wtr: &mut Writer<File>) {
    wtr.flush().unwrap();
}
