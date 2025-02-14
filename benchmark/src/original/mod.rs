use eyre::{anyhow as err, Result};
use polars::prelude::*;

mod simulate_price;

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

// let num_paths = 15000;
// let n_periods = 720;
// let cap_level = 0.3;
// let risk_free_rate = 0.05;

// let mut df = drop_nulls(&df, "TWAP_7d")?;

// let period_end_date_timestamp = df
//     .column("date")?
//     .datetime()?
//     .get(df.height() - 1)
//     .ok_or_else(|| err!("No row {} in the date column", df.height() - 1))?;

// let period_start_date_timestamp = df
//     .column("date")?
//     .datetime()?
//     .get(0)
//     .ok_or_else(|| err!("No row 0 in the date column"))?;

// let log_base_fee = compute_log_of_base_fees(&df)?;
// df.with_column(Series::new("log_base_fee".into(), log_base_fee))?;

// let (trend_model, trend_values) = discover_trend(&df)?;
// df.with_column(Series::new("trend".into(), trend_values))?;
// df.with_column(Series::new(
//     "detrended_log_base_fee".into(),
//     df["log_base_fee"].f64()? - df["trend"].f64()?,
// ))?;

// let (de_seasonalised_detrended_log_base_fee, season_param) =
//     remove_seasonality(&mut df, period_start_date_timestamp)?;
// df.with_column(Series::new(
//     "de_seasonalized_detrended_log_base_fee".into(),
//     de_seasonalised_detrended_log_base_fee.clone().to_vec(),
// ))?;

// let (de_seasonalized_detrended_simulated_prices, _params) = simulate_prices(
//     de_seasonalised_detrended_log_base_fee.view(),
//     n_periods,
//     num_paths,
//     max_iterations,
// )?;

// let total_hours = (period_end_date_timestamp - period_start_date_timestamp) / 3600 / 1000;
// let sim_hourly_times: Array1<f64> =
//     Array1::range(0.0, n_periods as f64, 1.0).mapv(|i| total_hours as f64 + i);

// let c = season_matrix(sim_hourly_times);
// let season = c.dot(&season_param);
// let season_reshaped = season.into_shape((n_periods, 1)).unwrap();

// let detrended_simulated_prices = &de_seasonalized_detrended_simulated_prices + &season_reshaped;

// let log_twap_7d: Vec<f64> = df
//     .column("TWAP_7d")?
//     .f64()?
//     .into_no_null_iter()
//     .map(|x| x.ln())
//     .collect();

// let returns: Vec<f64> = log_twap_7d
//     .windows(2)
//     .map(|window| window[1] - window[0])
//     .collect();
// let returns: Vec<f64> = returns.into_iter().filter(|&x| !x.is_nan()).collect();

// let mu = 0.05 / 52.0;
// let sigma = standard_deviation(returns) * f64::sqrt(24.0 * 7.0);
// let dt = 1.0 / 24.0;

// let mut stochastic_trend = Array2::<f64>::zeros((n_periods, num_paths));
// let normal = Normal::new(0.0, sigma * (f64::sqrt(dt))).unwrap();
// let mut rng = thread_rng();
// for i in 0..num_paths {
//     let random_shocks: Vec<f64> = (0..n_periods).map(|_| normal.sample(&mut rng)).collect();
//     let mut cumsum = 0.0;
//     for j in 0..n_periods {
//         cumsum += (mu - 0.5 * sigma.powi(2)) * dt + random_shocks[j];
//         stochastic_trend[[j, i]] = cumsum;
//     }
// }

// let coeffs = trend_model.params();
// let final_trend_value = {
//     let x = (df.height() - 1) as f64;
//     coeffs[0] * x + coeffs[1]
// };

// let mut simulated_log_prices = Array2::<f64>::zeros((n_periods, num_paths));
// for i in 0..n_periods {
//     let trend = final_trend_value;
//     for j in 0..num_paths {
//         simulated_log_prices[[i, j]] =
//             detrended_simulated_prices[[i, j]] + trend + stochastic_trend[[i, j]];
//     }
// }

// let simulated_prices = simulated_log_prices.mapv(f64::exp);
// let twap_start = n_periods.saturating_sub(24 * 7);
// let final_prices_twap = simulated_prices
//     .slice(s![twap_start.., ..])
//     .mean_axis(Axis(0))
//     .unwrap();

// let payoffs = final_prices_twap.mapv(|price| {
//     let capped_price = (1.0 + cap_level) * strike;
//     (price.min(capped_price) - strike).max(0.0)
// });

// let average_payoff = payoffs.mean().unwrap_or(0.0);
// let reserve_price = f64::exp(-risk_free_rate) * average_payoff;

// Ok(reserve_price)
