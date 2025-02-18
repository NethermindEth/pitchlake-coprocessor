use std::f64::consts::PI;

use chrono::DateTime;
use eyre::{anyhow as err, Result};
use linfa::prelude::*;
use linfa::traits::Fit;
use linfa_linear::{FittedLinearRegression, LinearRegression};
use ndarray::prelude::*;
use ndarray::{stack, Array1, Array2, Axis};
use ndarray_linalg::LeastSquaresSvd;
use ndarray_rand::rand_distr::Normal;
use polars::prelude::*;
use rand::thread_rng;
use rand_distr::Distribution;
use simulate_price::simulate_prices;

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

// Computes the natural logarithm of 'base_fee' values
fn compute_log_of_base_fees(df: &DataFrame) -> Result<Vec<f64>> {
    let log_base_fees: Vec<f64> = df
        .column("base_fee")?
        .f64()?
        .into_no_null_iter()
        .map(|x| x.ln())
        .collect();
    Ok(log_base_fees)
}

fn discover_trend(df: &DataFrame) -> Result<(FittedLinearRegression<f64>, Vec<f64>)> {
    let time_index: Vec<f64> = (0..df.height() as i64).map(|i| i as f64).collect();

    let ones = Array::<f64, Ix1>::ones(df.height());
    let x = stack![Axis(1), Array::from(time_index.clone()), ones];

    let y = Array1::from(
        df["log_base_fee"]
            .f64()?
            .into_no_null_iter()
            .collect::<Vec<f64>>(),
    );

    let dataset = Dataset::<f64, f64, Ix1>::new(x.clone(), y);
    let trend_model = LinearRegression::default()
        .with_intercept(false)
        .fit(&dataset)?;

    let trend_values = trend_model.predict(&x).as_targets().to_vec();

    Ok((trend_model, trend_values))
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

fn season_matrix(t: Array1<f64>) -> Array2<f64> {
    let sin_2pi_24 = t.mapv(|time| (2.0 * PI * time / 24.0).sin());
    let cos_2pi_24 = t.mapv(|time| (2.0 * PI * time / 24.0).cos());
    let sin_4pi_24 = t.mapv(|time| (4.0 * PI * time / 24.0).sin());
    let cos_4pi_24 = t.mapv(|time| (4.0 * PI * time / 24.0).cos());
    let sin_8pi_24 = t.mapv(|time| (8.0 * PI * time / 24.0).sin());
    let cos_8pi_24 = t.mapv(|time| (8.0 * PI * time / 24.0).cos());
    let sin_2pi_24_7 = t.mapv(|time| (2.0 * PI * time / (24.0 * 7.0)).sin());
    let cos_2pi_24_7 = t.mapv(|time| (2.0 * PI * time / (24.0 * 7.0)).cos());
    let sin_4pi_24_7 = t.mapv(|time| (4.0 * PI * time / (24.0 * 7.0)).sin());
    let cos_4pi_24_7 = t.mapv(|time| (4.0 * PI * time / (24.0 * 7.0)).cos());
    let sin_8pi_24_7 = t.mapv(|time| (8.0 * PI * time / (24.0 * 7.0)).sin());
    let cos_8pi_24_7 = t.mapv(|time| (8.0 * PI * time / (24.0 * 7.0)).cos());

    stack![
        Axis(1),
        sin_2pi_24,
        cos_2pi_24,
        sin_4pi_24,
        cos_4pi_24,
        sin_8pi_24,
        cos_8pi_24,
        sin_2pi_24_7,
        cos_2pi_24_7,
        sin_4pi_24_7,
        cos_4pi_24_7,
        sin_8pi_24_7,
        cos_8pi_24_7
    ]
}

fn remove_seasonality(
    df: &mut DataFrame,
    start_date_timestamp: i64,
) -> Result<(Array1<f64>, Array1<f64>)> {
    let start_date = DateTime::from_timestamp(start_date_timestamp / 1000, 0)
        .ok_or_else(|| err!("Can't calculate the start date"))?;

    let t_series: Vec<f64> = df
        .column("date")?
        .datetime()?
        .into_iter()
        .map(|opt_date| {
            opt_date.map_or(0.0, |date| {
                (DateTime::from_timestamp(date / 1000, 0).unwrap() - start_date).num_seconds()
                    as f64
                    / 3600.0
            })
        })
        .collect();

    df.with_column(Series::new("t".into(), t_series))?;

    let t_array = df["t"].f64()?.to_ndarray()?.to_owned();
    let c = season_matrix(t_array);

    let detrended_log_base_fee_array = df["detrended_log_base_fee"].f64()?.to_ndarray()?.to_owned();
    let season_param = c.least_squares(&detrended_log_base_fee_array)?.solution;
    let season = c.dot(&season_param);
    let de_seasonalised_detrended_log_base_fee =
        df["detrended_log_base_fee"].f64()?.to_ndarray()?.to_owned() - season;

    Ok((de_seasonalised_detrended_log_base_fee, season_param))
}

fn standard_deviation(returns: Vec<f64>) -> f64 {
    let n = returns.len() as f64;
    if n < 2.0 {
        return 0.0; // Return 0 for vectors with less than 2 elements
    }
    let mean = returns.iter().sum::<f64>() / n;
    let variance = returns.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    variance.sqrt()
}

#[derive(Debug)]
pub struct AllInputsToReservePrice {
    pub season_param: Array1<f64>,
    pub de_seasonalised_detrended_log_base_fee: Array1<f64>,
    pub de_seasonalized_detrended_simulated_prices: Array2<f64>,
    pub twap_7d: Vec<f64>,
    pub slope: f64,
    pub intercept: f64,
    pub reserve_price: f64,
    pub positions: Vec<f64>,
}

pub fn calculate_reserve_price(inputs: &Vec<(i64, f64)>) -> AllInputsToReservePrice {
    let mut df = convert_input_to_df(inputs);

    let period_end_date_timestamp = df
        .column("date")
        .unwrap()
        .datetime()
        .unwrap()
        .get(df.height() - 1)
        .ok_or_else(|| err!("No row {} in the date column", df.height() - 1))
        .unwrap();

    let period_start_date_timestamp = df
        .column("date")
        .unwrap()
        .datetime()
        .unwrap()
        .get(0)
        .unwrap();

    let log_base_fee = compute_log_of_base_fees(&df).unwrap();
    df.with_column(Series::new("log_base_fee".into(), log_base_fee))
        .unwrap();

    let (trend_model, trend_values) = discover_trend(&df).unwrap();
    df.with_column(Series::new("trend".into(), trend_values))
        .unwrap();
    df.with_column(Series::new(
        "detrended_log_base_fee".into(),
        df["log_base_fee"].f64().unwrap() - df["trend"].f64().unwrap(),
    ))
    .unwrap();

    let (de_seasonalised_detrended_log_base_fee, season_param) =
        remove_seasonality(&mut df, period_start_date_timestamp).unwrap();
    df.with_column(Series::new(
        "de_seasonalized_detrended_log_base_fee".into(),
        de_seasonalised_detrended_log_base_fee.clone().to_vec(),
    ))
    .unwrap();

    let num_paths = 15000;
    let n_periods = 720;
    let cap_level = 0.3;
    let risk_free_rate = 0.05;
    let max_iterations = 2000;
    let (de_seasonalized_detrended_simulated_prices, positions) = simulate_prices(
        de_seasonalised_detrended_log_base_fee.view(),
        n_periods,
        num_paths,
        max_iterations,
    )
    .unwrap();

    let total_hours = (period_end_date_timestamp - period_start_date_timestamp) / 3600 / 1000;
    let sim_hourly_times: Array1<f64> =
        Array1::range(0.0, n_periods as f64, 1.0).mapv(|i| total_hours as f64 + i);

    let c = season_matrix(sim_hourly_times);
    let season = c.dot(&season_param);
    let season_reshaped = season.into_shape((n_periods, 1)).unwrap();

    let detrended_simulated_prices = &de_seasonalized_detrended_simulated_prices + &season_reshaped;

    let twap: Vec<f64> = df
        .column("TWAP_7d")
        .unwrap()
        .f64()
        .unwrap()
        .into_no_null_iter()
        .collect();

    let log_twap_7d: Vec<f64> = twap.iter().map(|x| x.ln()).collect();

    let returns: Vec<f64> = log_twap_7d
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect();
    let returns: Vec<f64> = returns.into_iter().filter(|&x| !x.is_nan()).collect();

    let mu = 0.05 / 52.0;
    let sigma = standard_deviation(returns) * f64::sqrt(24.0 * 7.0);
    let dt = 1.0 / 24.0;

    let mut stochastic_trend = Array2::<f64>::zeros((n_periods, num_paths));
    let normal = Normal::new(0.0, sigma * (f64::sqrt(dt))).unwrap();
    let mut rng = thread_rng();
    for i in 0..num_paths {
        let random_shocks: Vec<f64> = (0..n_periods).map(|_| normal.sample(&mut rng)).collect();
        let mut cumsum = 0.0;
        for j in 0..n_periods {
            cumsum += (mu - 0.5 * sigma.powi(2)) * dt + random_shocks[j];
            stochastic_trend[[j, i]] = cumsum;
        }
    }

    let coeffs = trend_model.params();
    let final_trend_value = {
        let x = (df.height() - 1) as f64;
        coeffs[0] * x + coeffs[1]
    };

    let mut simulated_log_prices = Array2::<f64>::zeros((n_periods, num_paths));
    for i in 0..n_periods {
        let trend = final_trend_value;
        for j in 0..num_paths {
            simulated_log_prices[[i, j]] =
                detrended_simulated_prices[[i, j]] + trend + stochastic_trend[[i, j]];
        }
    }

    let simulated_prices = simulated_log_prices.mapv(f64::exp);
    let twap_start = n_periods.saturating_sub(24 * 7);
    let final_prices_twap = simulated_prices
        .slice(s![twap_start.., ..])
        .mean_axis(Axis(0))
        .unwrap();

    let twap_7d_series = df.column("TWAP_7d").unwrap();
    let strike = twap_7d_series
        .f64()
        .unwrap()
        .last()
        .ok_or_else(|| err!("The series is empty"))
        .unwrap();
    let payoffs = final_prices_twap.mapv(|price| {
        let capped_price = (1.0 + cap_level) * strike;
        (price.min(capped_price) - strike).max(0.0)
    });

    let average_payoff = payoffs.mean().unwrap_or(0.0);
    let reserve_price = f64::exp(-risk_free_rate) * average_payoff;

    AllInputsToReservePrice {
        season_param,
        de_seasonalised_detrended_log_base_fee,
        de_seasonalized_detrended_simulated_prices,
        positions,
        twap_7d: twap,
        slope: trend_model.params()[0],
        intercept: trend_model.params()[1],
        reserve_price,
    }
}
