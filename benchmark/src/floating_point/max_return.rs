// fn add_twap_30d(df: DataFrame) -> Result<DataFrame> {
//     let required_window_size = 24 * 30;

//     if df.height() < required_window_size {
//         return Err(err!(
//             "Insufficient data: At least {} data points are required, but only {} provided.",
//             required_window_size,
//             df.height()
//         ));
//     }

//     let lazy_df = df.lazy().with_column(
//         col("base_fee")
//             .rolling_mean(RollingOptionsFixedWindow {
//                 window_size: required_window_size,
//                 min_periods: 1,
//                 weights: None,
//                 center: false,
//                 fn_params: None,
//             })
//             .alias("TWAP_30d"),
//     );

//     let df = lazy_df.collect()?;

//     Ok(df.fill_null(FillNullStrategy::Backward(None))?)
// }
use eyre::{anyhow as err, Result};
use nalgebra::DVector;

fn add_twap_30d(data: &Vec<(i64, f64)>) -> Result<Vec<f64>> {
    let required_window_size = 24 * 30;

    // TODO: can be refactored with add_twap_7d
    let n = data.len();

    if n < required_window_size {
        return Err(err!(
            "Insufficient data: At least {} data points are required, but only {} provided.",
            required_window_size,
            n
        ));
    }

    let values = DVector::from_iterator(n, data.iter().map(|&(_, value)| value));
    let mut twap_values = Vec::with_capacity(n);

    for i in 0..n {
        let window_start = if i >= required_window_size {
            i - required_window_size + 1
        } else {
            0
        };
        let window_mean = values.rows(window_start, i - window_start + 1).mean();
        twap_values.push(window_mean);
    }

    Ok(twap_values)
}


fn calculate_30d_returns(twap_30d: &Vec<f64>) -> Result<Vec<f64>> {
    // 24 hours * 30 days = 720 hours
    let period = 24 * 30;

    if twap_30d.len() <= period {
        return Err(err!("Input vector must be longer than {} elements", period));
    }

    let mut returns = Vec::with_capacity(twap_30d.len());

    // Fill initial values with 0.0 since we can't calculate returns yet
    for _ in 0..period {
        returns.push(0.0);
    }

    // Calculate returns for remaining values
    for i in period..twap_30d.len() {
        let current = twap_30d[i];
        let previous = twap_30d[i - period];
        let return_value = (current / previous) - 1.0;
        returns.push(return_value);
    }

    Ok(returns)
}

// fn calculate_30d_returns(df: DataFrame) -> Result<DataFrame> {
//     // 24 hours * 30 days = 720 hours
//     let period = 24 * 30;

//     let df = df
//         .lazy()
//         .with_column(
//             (col("TWAP_30d") / col("TWAP_30d").shift(lit(period)) - lit(1.0)).alias("30d_returns"),
//         )
//         .collect()?;

//     Ok(df)
// }
