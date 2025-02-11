use chrono::prelude::*;
use chrono::Months;
use eyre::{anyhow as err, Error, Result};
use polars::prelude::*;

pub mod csv;

pub fn read_data_from_file(file_name: &str) -> DataFrame {
    let df: DataFrame = read_csv(file_name).expect("Cannot read file");
    df
}

pub fn add_df_property(df: DataFrame) -> DataFrame {
    let mut df = df;
    df = replace_timestamp_with_date(df).unwrap();
    df = group_by_1h_intervals(df).unwrap();
    df = add_twap_7d(df).unwrap();

    df
}

/// Adds a Time-Weighted Average Price (TWAP) column to the DataFrame.
///
/// This function calculates the 7-day TWAP for the 'base_fee' column and adds it as a new column
/// named 'TWAP_7d' to the input DataFrame.
///
/// # Arguments
///
/// * `df` - The input DataFrame containing the 'base_fee' column.
///
/// # Returns
///
/// A `Result` containing the DataFrame with the added 'TWAP_7d' column, or an `Error` if the
/// operation fails.
///
/// # Errors
///
/// This function will return an error if:
/// * The rolling mean calculation fails.
/// * The final collection of the lazy DataFrame fails.
///
fn add_twap_7d(df: DataFrame) -> Result<DataFrame> {
    let required_window_size = 24 * 7;

    tracing::debug!("DataFrame shape before TWAP: {:?}", df.shape());

    if df.height() < required_window_size {
        return Err(err!(
            "Insufficient data: At least {} data points are required, but only {} provided.",
            required_window_size,
            df.height()
        ));
    }

    let lazy_df = df.lazy().with_column(
        col("base_fee")
            .rolling_mean(RollingOptionsFixedWindow {
                window_size: required_window_size,
                min_periods: 1,
                weights: None,
                center: false,
                fn_params: None,
            })
            .alias("TWAP_7d"),
    );

    let df = lazy_df.collect()?;
    tracing::debug!("DataFrame shape after TWAP: {:?}", df.shape());

    Ok(df.fill_null(FillNullStrategy::Backward(None))?)
}
/// Groups the DataFrame by 1-hour intervals and aggregates specified columns.
///
/// This function takes a DataFrame and groups it by 1-hour intervals based on the 'date' column.
/// It then calculates the mean values for 'base_fee', 'gas_limit', 'gas_used', and 'number' columns
/// within each interval.
///
/// # Arguments
///
/// * `df` - The input DataFrame to be grouped and aggregated.
///
/// # Returns
///
/// A `Result` containing the grouped and aggregated DataFrame, or an `Error` if the operation fails.
///
/// # Errors
///
/// This function will return an error if:
/// * The grouping or aggregation operations fail.
/// * The final collection of the lazy DataFrame fails.
///
fn group_by_1h_intervals(df: DataFrame) -> Result<DataFrame, Error> {
    let df = df
        .lazy()
        .group_by_dynamic(
            col("date"),
            [],
            DynamicGroupOptions {
                every: Duration::parse("1h"),
                period: Duration::parse("1h"),
                offset: Duration::parse("0"),
                ..Default::default()
            },
        )
        .agg([
            col("base_fee").mean(),
            col("gas_limit").mean(),
            col("gas_used").mean(),
            col("number").mean(),
        ])
        .collect()?;

    Ok(df)
}

pub fn convert_to_timestamp_base_fee_tuple(df: DataFrame) -> Vec<(i64, f64)> {
    let xxxx = df.select(["date", "base_fee"]).unwrap();

    let dates = xxxx.column("date").unwrap().datetime().unwrap();
    let base_fee = xxxx.column("base_fee").unwrap().f64().unwrap();
    let tuples: Vec<(i64, f64)> = dates
        .iter()
        .zip(base_fee.iter())
        .map(|(d, g)| (d.unwrap() / 1000, g.unwrap()))
        .collect();

    tuples
}

/// Replaces the 'timestamp' column with a 'date' column in a DataFrame.
///
/// This function takes a DataFrame with a 'timestamp' column, converts the timestamps
/// to milliseconds, casts them to datetime, and replaces the 'timestamp' column with
/// a new 'date' column.
///
/// # Arguments
///
/// * `df` - A mutable reference to the input DataFrame.
///
/// # Returns
///
/// A `Result` containing the modified DataFrame with the 'timestamp' column replaced
/// by the 'date' column, or an `Error` if the operation fails.
///
/// # Errors
///
/// This function will return an error if:
/// * The 'timestamp' column is missing or cannot be accessed.
/// * The conversion to milliseconds or casting to datetime fails.
/// * The column replacement or renaming operations fail.
///
fn replace_timestamp_with_date(mut df: DataFrame) -> Result<DataFrame, Error> {
    let dates = df
        .column("timestamp")?
        .i64()?
        .apply(|s| s.map(|s| s * 1000)) // convert into milliseconds
        .into_series()
        .cast(&DataType::Datetime(TimeUnit::Milliseconds, None))?;

    df.replace("timestamp", dates)?;
    df.rename("timestamp", "date".into())?;

    Ok(df)
}

fn read_csv(file: &str) -> PolarsResult<DataFrame> {
    CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(file.into()))?
        .finish()
}

// group the data into x number of months per period
pub fn split_dataframe_into_periods(
    df: DataFrame,
    period_length_in_months: i32,
) -> Result<Vec<DataFrame>, Error> {
    let mut period_dataframes: Vec<DataFrame> = Vec::new();

    let start_date_value = df
        .column("date")?
        .datetime()?
        .get(0)
        .ok_or_else(|| err!("No row 0 in the date column"))?;
    let start_date = DateTime::from_timestamp(start_date_value / 1000, 0)
        .ok_or_else(|| err!("Can't calculate the start date"))?;

    let end_date_row = df.height() - 1;
    let end_date_value = df
        .column("date")?
        .datetime()?
        .get(end_date_row)
        .ok_or_else(|| err!("No row {end_date_row} in the date column"))?;
    let end_date = DateTime::from_timestamp(end_date_value / 1000, 0)
        .ok_or_else(|| err!("Can't calculate the end date"))?;

    let num_months = (end_date.year() - start_date.year()) * 12 + i32::try_from(end_date.month())?
        - i32::try_from(start_date.month())?
        + 1;

    for i in 0..num_months - (period_length_in_months - 1) {
        let period_start = start_date + Months::new(i as u32);
        let period_end = period_start + Months::new(period_length_in_months as u32);
        let period_df = df
            .clone()
            .lazy()
            .filter(
                col("date")
                    .gt_eq(lit(period_start.naive_utc()))
                    .and(col("date").lt(lit(period_end.naive_utc()))),
            )
            .collect()?;

        period_dataframes.push(period_df);
    }

    Ok(period_dataframes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_dataframe_into_periods() {
        let df = read_data_from_file("../data.csv");
        let df = add_df_property(df);
        let periods = split_dataframe_into_periods(df, 3).unwrap();

        let xxxx = periods[0].column("date").unwrap().get(0).unwrap();
        let xxxx_last = periods[0]
            .column("date")
            .unwrap()
            .get(periods[0].height() - 1)
            .unwrap();

        let yyyy = periods[1].column("date").unwrap().get(0).unwrap();
        let yyyy_last = periods[1]
            .column("date")
            .unwrap()
            .get(periods[1].height() - 1)
            .unwrap();

        let zzzz = periods[2].column("date").unwrap().get(0).unwrap();
        let zzzz_last = periods[2]
            .column("date")
            .unwrap()
            .get(periods[2].height() - 1)
            .unwrap();

        let aaaa = periods[3].column("date").unwrap().get(0).unwrap();
        let aaaa_last = periods[3]
            .column("date")
            .unwrap()
            .get(periods[3].height() - 1)
            .unwrap();

        println!("{:?}", xxxx);
        println!("{:?}", xxxx_last);
        println!("{:?}", yyyy);
        println!("{:?}", yyyy_last);

        // assert_eq!(periods.len(), 3);
    }

    #[test]
    fn test_convert_to_timestamp_gas_used_tuple() {
        let df = read_data_from_file("../data.csv");
        let df = add_df_property(df);
        let tuples = convert_to_timestamp_base_fee_tuple(df);
        println!("{:?}", tuples);
    }
}
