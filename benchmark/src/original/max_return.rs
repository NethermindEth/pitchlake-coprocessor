use eyre::{anyhow as err, Result};
use polars::prelude::*;

// df should aldy have average hourly base fee
pub fn calculate_max_return(df: DataFrame) -> Result<f64> {
    let mut df = add_twap_30d(df)?;
    df = drop_nulls(&df, "TWAP_30d")?;
    df = calculate_30d_returns(df)?;
    df = drop_nulls(&df, "30d_returns")?;

    let max_return = df
        .column("30d_returns")?
        .f64()?
        .max()
        .ok_or_else(|| err!("30d returns series is empty"))?;

    Ok(max_return)
}

pub fn calculate_30d_returns(df: DataFrame) -> Result<DataFrame> {
    // 24 hours * 30 days = 720 hours
    let period = 24 * 30;

    let df = df
        .lazy()
        .with_column(
            (col("TWAP_30d") / col("TWAP_30d").shift(lit(period)) - lit(1.0)).alias("30d_returns"),
        )
        .collect()?;

    Ok(df)
}

pub fn add_twap_30d(df: DataFrame) -> Result<DataFrame> {
    let required_window_size = 24 * 30;

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
            .alias("TWAP_30d"),
    );

    let df = lazy_df.collect()?;

    Ok(df.fill_null(FillNullStrategy::Backward(None))?)
}

fn drop_nulls(df: &DataFrame, column_name: &str) -> Result<DataFrame> {
    let df = df
        .clone()
        .lazy()
        .filter(col(column_name).is_not_null())
        .collect()?;

    Ok(df)
}
