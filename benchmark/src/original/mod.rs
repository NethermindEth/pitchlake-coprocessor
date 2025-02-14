use eyre::{anyhow as err, Result};
use polars::prelude::*;

fn add_twap_7d(df: DataFrame) -> Result<DataFrame> {
    let required_window_size = 24 * 7;

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

    Ok(df.fill_null(FillNullStrategy::Backward(None))?)
}

pub fn convert_input_to_df(inputs: &Vec<(i64, f64)>) -> DataFrame {
    let timestamps = inputs
        .iter()
        .map(|(timestamp, _)| *timestamp)
        .collect::<Vec<i64>>();
    let base_fees = inputs
        .iter()
        .map(|(_, base_fee)| *base_fee)
        .collect::<Vec<f64>>();
    let mut df = DataFrame::new(vec![
        Series::new("timestamp".into(), timestamps),
        Series::new("base_fee".into(), base_fees),
    ])
    .unwrap();

    let dates = df
        .column("timestamp")
        .unwrap()
        .i64()
        .unwrap()
        .apply(|s| s.map(|s| s * 1000)) // convert into milliseconds
        .into_series()
        .cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
        .unwrap();

    df.replace("timestamp", dates).unwrap();
    df.rename("timestamp", "date".into()).unwrap();

    df = add_twap_7d(df).unwrap();

    df
}