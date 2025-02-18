use crate::is_saddle_point;

use super::gradient;
use eyre::Result;
use nalgebra::{DMatrix, DVector};
use rand::thread_rng;
use rand_distr::Distribution;
use statrs::distribution::{Binomial, Normal};

fn verify_minimize_result(
    initial_position: &Vec<f64>,
    pt: &DVector<f64>,
    pt_1: &DVector<f64>,
    gradient_tolerance: f64,
) -> bool {
    let gradient = gradient(&initial_position, pt, pt_1);
    is_saddle_point(&gradient, gradient_tolerance)
}

fn post_minimize_after_verify(
    positions: &Vec<f64>,
    de_seasonalised_detrended_log_base_fee: &DVector<f64>,
    n_periods: usize,
    num_paths: usize,
) -> Result<DMatrix<f64>> {
    let dt = 1.0 / (365.0 * 24.0);

    let alpha = positions[0] / dt;
    let kappa = (1.0 - positions[1]) / dt;
    let mu_j = positions[2];
    let sigma = (positions[3] / dt).sqrt();
    let sigma_j = positions[4].sqrt();
    let lambda_ = positions[5] / dt;

    println!("alpha: {}", alpha);
    println!("kappa: {}", kappa);
    println!("mu_j: {}", mu_j);
    println!("sigma: {}", sigma);
    println!("sigma_j: {}", sigma_j);
    println!("lambda_: {}", lambda_);

    // RNG for stochastic processes
    let mut rng = thread_rng();

    // Simulate the Poisson process (jumps)
    let binom = Binomial::new(lambda_ * dt, 1)?;
    let mut jumps = DMatrix::zeros(n_periods, num_paths);
    for i in 0..n_periods {
        for j in 0..num_paths {
            jumps[(i, j)] = binom.sample(&mut rng) as f64;
        }
    }

    // Initialize simulated prices
    let mut simulated_prices = DMatrix::zeros(n_periods, num_paths);
    let initial_price =
        de_seasonalised_detrended_log_base_fee[de_seasonalised_detrended_log_base_fee.len() - 1];
    for j in 0..num_paths {
        simulated_prices[(0, j)] = initial_price;
    }

    // Generate standard normal variables
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut n1 = DMatrix::zeros(n_periods, num_paths);
    let mut n2 = DMatrix::zeros(n_periods, num_paths);
    for i in 0..n_periods {
        for j in 0..num_paths {
            n1[(i, j)] = normal.sample(&mut rng);
            n2[(i, j)] = normal.sample(&mut rng);
        }
    }
    // Simulate prices over time
    for i in 1..n_periods {
        for j in 0..num_paths {
            let prev_price = simulated_prices[(i - 1, j)];
            let current_n1 = n1[(i, j)];
            let current_n2 = n2[(i, j)];
            let current_j = jumps[(i, j)];

            simulated_prices[(i, j)] = alpha * dt
                + (1.0 - kappa * dt) * prev_price
                + sigma * dt.sqrt() * current_n1
                + current_j * (mu_j + sigma_j * current_n2);
        }
    }

    Ok(simulated_prices)
}

pub fn simulate_price_verify_position(
    positions: &Vec<f64>,
    pt: &DVector<f64>,
    pt_1: &DVector<f64>,
    gradient_tolerance: f64,
    de_seasonalised_detrended_log_base_fee: &DVector<f64>,
    n_periods: usize,
    num_paths: usize,
) -> (bool, DMatrix<f64>) {
    let is_saddle_point = verify_minimize_result(positions, pt, pt_1, gradient_tolerance);

    let simulated_prices = post_minimize_after_verify(
        positions,
        de_seasonalised_detrended_log_base_fee,
        n_periods,
        num_paths,
    )
    .unwrap();

    return (is_saddle_point, simulated_prices);
}
