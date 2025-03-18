use eyre::{anyhow as err, Result};
use nalgebra::{DMatrix, DVector};
use rand::thread_rng;
use rand_distr::Distribution;
use statrs::distribution::Normal;

use super::season_matrix;

fn standard_deviation(values: &[f64]) -> f64 {
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    variance.sqrt()
}

pub fn calculate_reserve_price(
    period_start_timestamp: i64, // this timestamps does not include the nulled twap timestamps
    period_end_timestamp: i64,   // this timestamps does not include the nulled twap timestamps
    season_param: &DVector<f64>,
    de_seasonalized_detrended_simulated_prices: &DMatrix<f64>,
    twap_7d: &[f64],
    slope: f64,
    intercept: f64,
    log_base_fee_len: usize,
    num_paths: usize,
    n_periods: usize,
) -> Result<f64> {
    // timestamps are assumed to be in milliseconds for this calculation
    let total_hours = (period_end_timestamp * 1000 - period_start_timestamp * 1000) / 3600 / 1000;

    let sim_hourly_times = DVector::from_iterator(
        n_periods,
        (0..n_periods).map(|i| total_hours as f64 + i as f64),
    );

    let c = season_matrix(sim_hourly_times);
    let season = &c * season_param;
    let season_matrix = season.reshape_generic(nalgebra::Dyn(n_periods), nalgebra::Const::<1>);
    let season_matrix_shaped =
        DMatrix::from_fn(n_periods, num_paths, |row, _| season_matrix[(row, 0)]);

    let detrended_simulated_prices =
        de_seasonalized_detrended_simulated_prices + &season_matrix_shaped;

    let log_twap_7d: Vec<f64> = twap_7d.iter().map(|x| x.ln()).collect();
    let returns: Vec<f64> = log_twap_7d
        .windows(2)
        .map(|window| window[1] - window[0])
        .collect();

    let mu = 0.05 / 52.0;
    let sigma = standard_deviation(&returns) * f64::sqrt(24.0 * 7.0);
    let dt = 1.0 / 24.0;

    let mut stochastic_trend = DMatrix::zeros(n_periods, num_paths);
    let normal = Normal::new(0.0, sigma * f64::sqrt(dt))?;
    let mut rng = thread_rng();

    for i in 0..num_paths {
        let random_shocks: Vec<f64> = (0..n_periods).map(|_| normal.sample(&mut rng)).collect();
        let mut cumsum = 0.0;
        for j in 0..n_periods {
            cumsum += (mu - 0.5 * sigma.powi(2)) * dt + random_shocks[j];
            stochastic_trend[(j, i)] = cumsum;
        }
    }

    let final_trend_value = slope * (log_base_fee_len - 1) as f64 + intercept;
    let mut simulated_log_prices = DMatrix::zeros(n_periods, num_paths);

    for i in 0..n_periods {
        let trend = final_trend_value;
        for j in 0..num_paths {
            simulated_log_prices[(i, j)] =
                detrended_simulated_prices[(i, j)] + trend + stochastic_trend[(i, j)];
        }
    }

    let simulated_prices = simulated_log_prices.map(f64::exp);

    let twap_start = n_periods.saturating_sub(24 * 7);

    let final_prices_twap = simulated_prices
        .rows(twap_start, n_periods - twap_start)
        .row_mean();

    let strike = twap_7d.last().ok_or_else(|| err!("The series is empty"))?;
    let capped_price = (1.0 + 0.3) * strike;
    let payoffs = final_prices_twap.map(|price| (price.min(capped_price) - strike).max(0.0));
    let average_payoff = payoffs.mean();

    let reserve_price = f64::exp(-0.05) * average_payoff;

    Ok(reserve_price)
}

pub fn calculated_reserve_price_from_simulated_log_prices(
    simulated_log_prices: &DMatrix<f64>,
    twap_7d: &[f64],
    n_periods: usize,
) -> Result<f64> {
    let simulated_prices = simulated_log_prices.map(f64::exp);
    let twap_start = n_periods.saturating_sub(24 * 7);

    let final_prices_twap = simulated_prices
        .rows(twap_start, n_periods - twap_start)
        .row_mean();

    let strike = twap_7d.last().ok_or_else(|| err!("The series is empty"))?;
    let capped_price = (1.0 + 0.3) * strike;
    let payoffs = final_prices_twap.map(|price| (price.min(capped_price) - strike).max(0.0));
    let average_payoff = payoffs.mean();

    let reserve_price = f64::exp(-0.05) * average_payoff;

    Ok(reserve_price)
}
