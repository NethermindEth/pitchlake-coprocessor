use std::fs::File;

use csv::Writer;

pub fn open_csv_writer(file_name: &str) -> Writer<File> {
    let wtr = Writer::from_path(file_name).unwrap();
    wtr
}

pub fn write_to_csv(wtr: &mut Writer<File>, data: &[String]) {
    wtr.write_record(data).unwrap();
}

pub fn open_error_bound_diff_csv_writer(file_name: &str) -> Writer<File> {
    let mut wtr = Writer::from_path(file_name).unwrap();
    write_to_csv(
        &mut wtr,
        &[
            "i".to_owned(),
            "j".to_owned(),
            "target_val".to_owned(),
            "calc_val".to_owned(),
            "percentage_diff".to_owned(),
        ],
    );
    wtr
}

pub fn write_error_bound_diff_to_csv(
    wtr: &mut Writer<File>,
    i: usize,
    j: usize,
    target_val: f64,
    calc_val: f64,
    percentage_diff: f64,
) {
    write_to_csv(
        wtr,
        &[
            i.to_string(),
            j.to_string(),
            target_val.to_string(),
            calc_val.to_string(),
            percentage_diff.to_string(),
        ],
    );
}

pub fn close_csv_file(wtr: &mut Writer<File>) {
    wtr.flush().unwrap();
}
