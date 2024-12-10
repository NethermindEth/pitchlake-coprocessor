use eth_rlp_types::BlockHeader;
// use chrono::{prelude::*, Duration};
use eyre::{anyhow as err, Result};
use nalgebra::{DMatrix, OVector};
use ndarray::{prelude::*, OwnedRepr, ViewRepr};
use ndarray::{stack, Array1, Array2, Axis};
use ndarray_rand::rand_distr::Normal;
use optimization::{Func, GradientDescent, Minimizer, NumericalDifferentiation};
use rand::prelude::*;
use rand_distr::Distribution;
use statrs::distribution::Binomial;
use std::f64::consts::PI;
// use std::error::Error;
// use chrono::{DateTime};

use super::utils::hex_string_to_f64;

pub fn calculate_reserve_price(block_headers: Vec<BlockHeader>) -> Result<f64> {
    if block_headers.is_empty() {
        // tracing::error!("No block headers provided.");
        return Err(eyre::eyre!("No block headers provided."));
    }

    // tracing::info!("Processing block headers...");
    let mut data = Vec::new();

    for header in block_headers {
        let timestamp: i64 = header.timestamp.ok_or_else(|| err!("No timestamp in header"))?.parse().expect("Unable to parse timestamp");
        let base_fee = hex_string_to_f64(
            &header.base_fee_per_gas.ok_or_else(|| err!("No base fee in header"))?,
        )?;
        // tracing::debug!("Timestamp: {}, Base Fee: {}", timestamp, base_fee); // Print timestamp and base fee
        data.push((timestamp * 1000, base_fee));
    }

    // tracing::info!("Sorting data by timestamp...");
    data.sort_by(|a, b| a.0.cmp(&b.0));

    // tracing::info!("Calculating 7-day TWAP...");
    let twap_7d: Vec<f64> = add_twap_7d(&data)?;
    let strike = twap_7d.last().ok_or_else(|| {
        // tracing::error!("The series is empty.");
        err!("The series is empty")
    })?;
    // tracing::debug!("7-day TWAP: {:?}", twap_7d); // Print 7-day TWAP for debugging

    let num_paths = 15000;
    let n_periods = 720;

    let fees: Vec<&f64> = data.iter().map(|x| &x.1).collect();

    // tracing::info!("Computing log base fees...");
    let log_base_fee = compute_log_of_base_fees(&fees)?;
    // tracing::debug!("Log Base Fee: {:?}", log_base_fee); // Debug log base fee values

    // tracing::info!("Discovering trend...");
    // let (trend_model, trend_values) = discover_trend(&log_base_fee)?;
    let (slope, intercept, trend_values) = discover_trend(&log_base_fee)?;
    // tracing::debug!("Trend Model: {:?}", trend_model); // Debug trend model
    // tracing::debug!("Trend Values: {:?}", trend_values); // Debug trend values

    // tracing::info!("Computing detrended log base fee...");
    let detrended_log_base_fee: Vec<f64> = log_base_fee.iter()
        .zip(&trend_values)
        .map(|(log_base_fee, trend)| log_base_fee - trend)
        .collect();
    // tracing::debug!("Detrended Log Base Fee: {:?}", detrended_log_base_fee);

    // tracing::info!("Removing seasonality...");
    let (de_seasonalised_detrended_log_base_fee, season_param) = remove_seasonality(
        &detrended_log_base_fee, 
        &data
    )?;
    // tracing::debug!("Seasonal Params: {:?}", season_param);
    // tracing::debug!("de_seasonalised_detrended_log_base_fee: {:?}", de_seasonalised_detrended_log_base_fee);


    // tracing::info!("Simulating prices...");
    let (de_seasonalized_detrended_simulated_prices, _params) = simulate_prices(
        de_seasonalised_detrended_log_base_fee.view(),
        n_periods,
        num_paths,
    )?;
    // Nans
    // tracing::debug!("Simulated Prices: {:?}", de_seasonalized_detrended_simulated_prices);

    let period_start_timestamp = data[0].0;
    let period_end_timestamp = data.last().ok_or_else(|| err!("Missing end timestamp"))?.0;
    let total_hours = (period_end_timestamp - period_start_timestamp) / 3600 / 1000;
    // tracing::debug!("Period start: {}, Period end: {}, Total hours: {}", period_start_timestamp, period_end_timestamp, total_hours);

    // tracing::info!("Generating seasonal component matrix...");
    let sim_hourly_times: Array1<f64> = Array1::range(0.0, n_periods as f64, 1.0).mapv(|i| total_hours as f64 + i);
    // tracing::debug!("Sim Hourly Times: {:?}", sim_hourly_times);
    let c = season_matrix(sim_hourly_times);
    let season = c.dot(&season_param);
    let season_reshaped = season.into_shape((n_periods, 1)).unwrap();
    // tracing::debug!("Season: {:?}", season_reshaped);

    // tracing::info!("Adding seasonal component...");
    let detrended_simulated_prices = &de_seasonalized_detrended_simulated_prices + &season_reshaped;

    // tracing::info!("Calculating log returns from TWAP 7d...");
    let log_twap_7d = twap_7d.iter().map(|x| x.ln()).collect::<Vec<f64>>();
    let returns: Vec<f64> = log_twap_7d.windows(2).map(|window| window[1] - window[0]).collect();
    // tracing::debug!("Log Returns: {:?}", returns);

    // tracing::info!("Computing mu and sigma for stochastic trend...");
    let mu = 0.05 / 52.0;
    let sigma = standard_deviation(&returns) * f64::sqrt(24.0 * 7.0);
    let dt = 1.0 / 24.0;

    // Debugging mu and sigma values before simulating stochastic trend
    // tracing::debug!("mu: {}, sigma: {}", mu, sigma); 

    // tracing::info!("Simulating stochastic trend...");
    let mut stochastic_trend = Array2::<f64>::zeros((n_periods, num_paths));
    let normal = Normal::new(0.0, sigma * (f64::sqrt(dt))).unwrap();
    let mut rng = thread_rng();

    // Debugging parameters before simulation
    // tracing::debug!("Simulating stochastic trend for {} periods and {} paths", n_periods, num_paths);

    for i in 0..num_paths {
        let random_shocks: Vec<f64> = (0..n_periods).map(|_| normal.sample(&mut rng)).collect();
        let mut cumsum = 0.0;
        for j in 0..n_periods {
            cumsum += (mu - 0.5 * sigma.powi(2)) * dt + random_shocks[j];
            stochastic_trend[[j, i]] = cumsum;
        }
    }

    // tracing::info!("Applying final trend and computing simulated prices...");
    // let coeffs = trend_model.params();
    // tracing::debug!("Coeffs: {}", coeffs);
    // let final_trend_value = coeffs[0] * (log_base_fee.len() - 1) as f64 + coeffs[1];
    
    // tracing::debug!("Coeffs: {}, {}", slope, intercept);
    let final_trend_value = slope * (log_base_fee.len() - 1) as f64 + intercept;

    // Debugging the final trend value
    // tracing::debug!("Final trend value: {}", final_trend_value);

    let mut simulated_log_prices = Array2::<f64>::zeros((n_periods, num_paths));
    for i in 0..n_periods {
        let trend = final_trend_value;
        for j in 0..num_paths {
            simulated_log_prices[[i, j]] = detrended_simulated_prices[[i, j]] + trend + stochastic_trend[[i, j]];
        }
    }

    // tracing::info!("Converting log prices to actual prices...");
    // NaNs
    // tracing::debug!("Simulated log prices before exp: {:?}", simulated_log_prices.slice(s![.., ..]));

    let simulated_prices = simulated_log_prices.mapv(f64::exp);

    let twap_start = n_periods.saturating_sub(24 * 7);
    // tracing::debug!("Simulated prices for TWAP computation: {:?}", simulated_prices.slice(s![twap_start.., ..]));

    let final_prices_twap = simulated_prices.slice(s![twap_start.., ..]).mean_axis(Axis(0)).unwrap();
    // tracing::debug!("Final Prices TWAP: {:?}", final_prices_twap);

    // tracing::info!("Calculating payoffs...");
    let capped_price = (1.0 + 0.3) * strike; // cap_level = 0.3
    let payoffs = final_prices_twap.mapv(|price| (price.min(capped_price) - strike).max(0.0));
    // tracing::debug!("Capped price: {:?}", capped_price);
    // tracing::debug!("Payoffs: {:?}", payoffs);

    let average_payoff = payoffs.mean().unwrap_or(0.0);
    // tracing::info!("Average payoff: {}", average_payoff);

    // tracing::info!("Calculating reserve price...");
    let reserve_price = f64::exp(-0.05) * average_payoff; // risk_free_rate = 0.05
    // tracing::info!("Reserve price: {}", reserve_price);

    Ok(reserve_price)
}


/// Perform a simple linear regression fit using least squares.
fn fit_linear_regression(x: &[f64], y: &[f64]) -> Result<(f64, f64)> {
    if x.len() != y.len() {
        return Err(eyre::eyre!("Input arrays x and y must have the same length.").into());
    }

    let n = x.len() as f64;

    // Calculate the necessary sums
    let sum_x = x.iter().copied().sum::<f64>();
    let sum_y = y.iter().copied().sum::<f64>();
    let sum_xx = x.iter().map(|&xi| xi * xi).sum::<f64>();
    let sum_xy = x.iter().zip(y.iter()).map(|(&xi, &yi)| xi * yi).sum::<f64>();

    // Calculate the slope (m) and intercept (b) using the least squares formulas
    let denominator = n * sum_xx - sum_x * sum_x;
    if denominator == 0.0 {
        return Err(eyre::eyre!("Singular matrix: cannot fit a linear regression.").into());
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denominator;
    let intercept = (sum_y * sum_xx - sum_x * sum_xy) / denominator;

    Ok((slope, intercept))
}

/// Predict the y values based on the fitted model.
fn predict(x: &[f64], slope: f64, intercept: f64) -> Vec<f64> {
    x.iter().map(|&xi| slope * xi + intercept).collect()
}

/// Discover the trend using linear regression.
fn discover_trend(log_base_fee: &[f64]) -> Result<(f64, f64, Vec<f64>)> {
    let time_index: Vec<f64> = (0..log_base_fee.len() as i64).map(|i| i as f64).collect();
    
    // Fit the linear regression model
    let (slope, intercept) = fit_linear_regression(&time_index, log_base_fee)?;

    // Predict the trend values
    let trend_values = predict(&time_index, slope, intercept);

    Ok((slope, intercept, trend_values))
}


// Computes the natural logarithm of the base fee values
fn compute_log_of_base_fees(base_fees: &Vec<&f64>) -> Result<Vec<f64>> {
    let log_base_fees: Vec<f64> = base_fees.iter().map(|&x| x.ln()).collect();
    Ok(log_base_fees)
}


fn least_squares(detrended_log_base_fee: ArrayBase<ViewRepr<&f64>, Dim<[usize; 1]>>, c: ArrayBase<ViewRepr<&f64>, Dim<[usize; 2]>>) -> Result<ndarray::ArrayBase<OwnedRepr<f64>, ndarray::Dim<[usize; 1]>>> {
        let detrended_log_base_fee_array = OVector::<f64, nalgebra::Dyn>::from_row_slice(detrended_log_base_fee.as_slice().unwrap());
        
        let (rows, cols) = c.dim();
        let c_vec: Vec<f64> = c.iter().cloned().collect();
        let c_matrix = DMatrix::from_row_slice(rows, cols, &c_vec);

        let epsilon = 1e-14;

        let results: lstsq::Lstsq<f64, nalgebra::Dyn> = lstsq::lstsq(&c_matrix, &detrended_log_base_fee_array, epsilon).unwrap();

        Ok(Array::from_vec(results.solution.as_slice().to_vec()))
    }


fn remove_seasonality(
    detrended_log_base_fee: &[f64],
    data: &Vec<(i64, f64)>,
) -> Result<(Array1<f64>, Array1<f64>)> {
    // Calculate the start date from the timestamp
    let start_timestamp = data.first().ok_or_else(|| err!("Missing start timestamp"))?.0;

    let t_series: Vec<f64> = data.iter().map(|(timestamp, _)| (timestamp - start_timestamp) as f64 / 1000.0 / 3600.0).collect();
    
    // tracing::debug!("Time series 't' created: {:?}", t_series);

    // Create the season matrix
    let c = season_matrix(Array1::from(t_series));
    // tracing::debug!("Season matrix: {:?}", c);

    // Convert the detrended log base fee to an ndarray
    let detrended_log_base_fee_array = Array1::from(detrended_log_base_fee.to_vec());
    // tracing::debug!("Detrended log base fee removearray: {:?}", detrended_log_base_fee_array);

    // Perform least squares to find the seasonal parameters
    // let season_param = c.least_squares(&detrended_log_base_fee_array)?.solution;
    let season_param = least_squares(detrended_log_base_fee_array.view(), c.view())?;
    // tracing::debug!("Seasonal parameters (least squares solution): {:?}", season_param);

    // Calculate the seasonal component
    let season = c.dot(&season_param);
    // tracing::debug!("Seasonal component: {:?}", season);

    // De-seasonalize the detrended log base fee
    let de_seasonalised_detrended_log_base_fee = detrended_log_base_fee_array - season;
    // tracing::debug!("De-seasonalized detrended log base fee: {:?}", de_seasonalised_detrended_log_base_fee);

    Ok((de_seasonalised_detrended_log_base_fee, season_param))
}


fn simulate_prices(
    de_seasonalised_detrended_log_base_fee: ArrayView1<f64>,
    n_periods: usize,
    num_paths: usize,
) -> Result<(Array2<f64>, Vec<f64>)> {
    let dt = 1.0 / (365.0 * 24.0);

    // Debugging input data
    // tracing::debug!("dt: {}, n_periods: {}, num_paths: {}", dt, n_periods, num_paths);
    // tracing::debug!("de_seasonalised_detrended_log_base_fee: {:?}", de_seasonalised_detrended_log_base_fee);

    // Prepare time series data
    let pt = de_seasonalised_detrended_log_base_fee
        .slice(s![1..])
        .to_owned()
        .into_dimensionality()?;
    let pt_1 = de_seasonalised_detrended_log_base_fee
        .slice(s![..-1])
        .to_owned()
        .into_dimensionality()?;

    // tracing::debug!("pt: {:?}", pt);
    // tracing::debug!("pt_1: {:?}", pt_1);

    // Define and initialize the function for numerical differentiation
    let function =
        NumericalDifferentiation::new(Func(|x: &[f64]| neg_log_likelihood(x, &pt, &pt_1)));

    // Minimizer for the optimization problem
    let minimizer = GradientDescent::new().max_iterations(Some(2400));

    // Perform the minimization
    let var_pt = pt.var(0.0);
    // tracing::debug!("Initial variance of pt: {}", var_pt);

    let solution = minimizer.minimize(
        &function,
        vec![-3.928e-02, 2.873e-04, 4.617e-02, var_pt, var_pt, 0.2],
    );

    // Extract the optimized parameters
    let params = &solution.position;
    // [4] negative
    // tracing::debug!("Optimization solution: {:?}", params);

    let alpha = params[0] / dt;
    let kappa = (1.0 - params[1]) / dt;
    let mu_j = params[2];
    let sigma = (params[3] / dt).sqrt();
    // NaN
    let sigma_j = params[4].sqrt();
    let lambda_ = params[5] / dt;

    // Debugging optimized parameters
    // tracing::debug!("alpha: {}, kappa: {}, mu_j: {}, sigma: {}, sigma_j: {}, lambda_: {}", alpha, kappa, mu_j, sigma, sigma_j, lambda_);

    // RNG for stochastic processes
    let mut rng = thread_rng();

    // Simulate the Poisson process (jumps)
    let j: Array2<f64> = {
        let binom = Binomial::new(lambda_ * dt, 1)?;
        Array2::from_shape_fn((n_periods, num_paths), |_| binom.sample(&mut rng) as f64)
    };
    // tracing::debug!("Simulated jumps (j): {:?}", j);

    // Initialize simulated prices
    let mut simulated_prices = Array2::zeros((n_periods, num_paths));
    simulated_prices
        .slice_mut(s![0, ..])
        .assign(&Array1::from_elem(
            num_paths,
            de_seasonalised_detrended_log_base_fee
                [de_seasonalised_detrended_log_base_fee.len() - 1],
        ));

    // tracing::debug!("Initial simulated prices (t=0): {:?}", simulated_prices.slice(s![0, ..]));

    // Generate standard normal variables
    let normal = Normal::new(0.0, 1.0).unwrap();
    let n1 = Array2::from_shape_fn((n_periods, num_paths), |_| normal.sample(&mut rng));
    let n2 = Array2::from_shape_fn((n_periods, num_paths), |_| normal.sample(&mut rng));

    // Simulate prices over time
    for i in 1..n_periods {
        let prev_prices = simulated_prices.slice(s![i - 1, ..]);
        let current_n1 = n1.slice(s![i, ..]);
        let current_n2 = n2.slice(s![i, ..]);
        let current_j = j.slice(s![i, ..]);

        // Compute new prices
        let new_prices = &(alpha * dt
            + (1.0 - kappa * dt) * &prev_prices
            + sigma * dt.sqrt() * &current_n1
            + &current_j * (mu_j + sigma_j * &current_n2));

        simulated_prices
            .slice_mut(s![i, ..])
            .assign(&new_prices.clone());

        // Debugging price evolution at each step
        // tracing::debug!(
        //     "Simulated prices at step {}: {:?}",
        //     i,
        //     simulated_prices.slice(s![i, ..])
        // );
    }

    // Return the simulated prices and model parameters
    // tracing::info!("Finished simulating prices.");
    Ok((simulated_prices, params.to_vec()))
}


fn drop_nulls<T>(data: &mut Vec<T>) 
where
    T: PartialEq + Default,
{
    data.retain(|x| *x != T::default());
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

fn standard_deviation(values: &[f64]) -> f64 {
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    variance.sqrt()
}


/// Calculates the probability density function (PDF) for the Mean-Reverting Jump (MRJ) model.
///
/// This function computes the PDF of the MRJ model given the model parameters and observed prices.
///
/// # Arguments
///
/// * `params` - A slice of f64 values representing the model parameters:
///   [a, phi, mu_j, sigma_sq, sigma_sq_j, lambda]
/// * `pt` - An Array1<f64> of observed prices at time t
/// * `pt_1` - An Array1<f64> of observed prices at time t-1
///
/// # Returns
///
/// * `Array1<f64>` - The calculated PDF values
///
/// # Notes
///
/// The MRJ model combines a mean-reverting process with a jump component. The PDF is a mixture
/// of two normal distributions, weighted by the jump probability (lambda).
fn mrjpdf(params: &[f64], pt: &Array1<f64>, pt_1: &Array1<f64>) -> Array1<f64> {
    let (a, phi, mu_j, sigma_sq, sigma_sq_j, lambda) = (
        params[0], params[1], params[2], params[3], params[4], params[5],
    );

    let term1 = lambda
        * (-((pt - a - phi * pt_1 - mu_j).mapv(|x| x.powi(2))) / (2.0 * (sigma_sq + sigma_sq_j)))
            .mapv(f64::exp)
        / ((2.0 * std::f64::consts::PI * (sigma_sq + sigma_sq_j)).sqrt());

    let term2 = (1.0 - lambda)
        * (-((pt - a - phi * pt_1).mapv(|x| x.powi(2))) / (2.0 * sigma_sq)).mapv(f64::exp)
        / ((2.0 * std::f64::consts::PI * sigma_sq).sqrt());

    term1 + term2
}

/// Calculates the negative log-likelihood for the mean-reverting jump diffusion model.
///
/// This function computes the negative log-likelihood of the observed data given the model parameters.
/// It's used in parameter estimation for the mean-reverting jump diffusion model.
///
/// # Arguments
///
/// * `params` - A slice of f64 values representing the model parameters:
///   [a, phi, mu_j, sigma_sq, sigma_sq_j, lambda]
/// * `pt` - An Array1<f64> of observed prices at time t
/// * `pt_1` - An Array1<f64> of observed prices at time t-1
///
/// # Returns
///
/// * `f64` - The negative log-likelihood value
///
/// # Notes
///
/// The function adds a small constant (1e-10) to each PDF value before taking the logarithm
/// to avoid potential issues with zero values.
fn neg_log_likelihood(params: &[f64], pt: &Array1<f64>, pt_1: &Array1<f64>) -> f64 {
    let pdf_vals = mrjpdf(params, pt, pt_1);
    -pdf_vals.mapv(|x| (x + 1e-10).ln()).sum()
}

fn add_twap_7d(data: &Vec<(i64, f64)>) -> Result<Vec<f64>> {
    let required_window_size = 24 * 7;
    let n = data.len();
    
    if n < required_window_size {
        return Err(err!(
            "Insufficient data: At least {} data points are required, but only {} provided.",
            required_window_size,
            n
        ));
    }

    let mut twap_values = vec![0.0; n];

    for i in 0..n {
        let window_start = if i >= required_window_size { i - required_window_size + 1 } else { 0 };
        
        let window = &data[window_start..=i];
        let sum: f64 = window.iter().map(|&(_, value)| value).sum();
        let mean = sum / window.len() as f64;

        twap_values[i] = mean;
    }

    Ok(twap_values)
}
