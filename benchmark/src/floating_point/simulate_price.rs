use std::{f64::EPSILON, ops::Add};

use eyre::Result;
use nalgebra::{DMatrix, DVector};
use rand::thread_rng;
use rand_distr::Distribution;
use statrs::distribution::{Binomial, Normal};

use super::{neg_log_likelihood, solution::Solution};

pub fn function_value(position: &[f64], pt: &DVector<f64>, pt_1: &DVector<f64>) -> f64 {
    neg_log_likelihood(position, pt, pt_1)
}

pub fn gradient(position: &[f64], pt: &DVector<f64>, pt_1: &DVector<f64>) -> Vec<f64> {
    let mut x: Vec<_> = position.to_vec();
    let current = function_value(position, pt, pt_1);

    position
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, x_i)| {
            let h = if x_i == 0.0 {
                EPSILON * 1.0e10
            } else {
                (EPSILON * x_i.abs()).sqrt()
            };

            assert!(h.is_finite());

            x[i] = x_i + h;

            let forward = function_value(&x, pt, pt_1);

            x[i] = x_i;

            let d_i = (forward - current) / h;

            assert!(d_i.is_finite());

            d_i
        })
        .collect()
}

pub fn is_saddle_point(gradient: &[f64], tolerance: f64) -> bool {
    gradient.iter().all(|dx| dx.abs() <= tolerance)
}

// ArmijoLineSearch::new(0.5, 1.0, 0.5)
fn search(
    initial_position: &[f64],
    direction: &[f64],
    pt: &DVector<f64>,
    pt_1: &DVector<f64>,
) -> Vec<f64> {
    let initial_value = function_value(initial_position, pt, pt_1);
    let gradient = gradient(initial_position, pt, pt_1);
    let control_parameter = 0.5;
    let initial_step_width = 1.0;
    let decay_factor = 0.5;

    let m = gradient
        .iter()
        .zip(direction)
        .map(|(g, d)| g * d)
        .fold(0.0, Add::add);
    let t = -control_parameter * m;

    assert!(t > 0.0);

    let mut step_width = initial_step_width;

    loop {
        let position: Vec<_> = initial_position
            .iter()
            .cloned()
            .zip(direction)
            .map(|(x, d)| x + step_width * d)
            .collect();
        let value = function_value(&position, pt, pt_1);

        if value <= initial_value - step_width * t {
            return position;
        }

        step_width *= decay_factor;
    }
}

pub fn minimize(
    initial_position: Vec<f64>,
    pt: &DVector<f64>,
    pt_1: &DVector<f64>,
    max_iterations: u64,
) -> Solution<f64> {
    let mut position = initial_position;
    let mut value = function_value(&position, pt, pt_1);

    let gradient_tolerance = 1.0e-4;
    let mut iteration = 0;
    loop {
        let gradient = gradient(&position, pt, pt_1);

        if is_saddle_point(&gradient, gradient_tolerance) {
            return Solution::new(position, value);
        }

        let direction: Vec<_> = gradient.into_iter().map(|g| -g).collect();
        let iter_xs = search(&position, &direction, pt, pt_1);
        position = iter_xs;
        value = function_value(&position, pt, pt_1);
        iteration += 1;

        if iteration == max_iterations {
            return Solution::new(position, value);
        }
    }
}

pub fn pre_minimize(
    de_seasonalised_detrended_log_base_fee: &DVector<f64>,
) -> (DVector<f64>, DVector<f64>, f64) {
    // Prepare time series data
    let pt = DVector::from_row_slice(&de_seasonalised_detrended_log_base_fee.as_slice()[1..]);
    let pt_1 = DVector::from_row_slice(
        &de_seasonalised_detrended_log_base_fee.as_slice()
            [..de_seasonalised_detrended_log_base_fee.len() - 1],
    );

    // Perform the minimization
    let var_pt = pt.iter().map(|&x| x * x).sum::<f64>() / pt.len() as f64;

    (pt, pt_1, var_pt)
}

pub fn post_minimize(
    solution: &Solution<f64>,
    de_seasonalised_detrended_log_base_fee: &DVector<f64>,
    n_periods: usize,
    num_paths: usize,
) -> Result<(DMatrix<f64>, Vec<f64>)> {
    let dt = 1.0 / (365.0 * 24.0);

    // Extract the optimized parameters
    let params = &solution.position;

    let alpha = params[0] / dt;
    let kappa = (1.0 - params[1]) / dt;
    let mu_j = params[2];
    let sigma = (params[3] / dt).sqrt();
    let sigma_j = params[4].sqrt();
    let lambda_ = params[5] / dt;

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

    Ok((simulated_prices, params.to_vec()))
}

pub fn simulate_price(
    de_seasonalised_detrended_log_base_fee: &DVector<f64>,
) -> (DMatrix<f64>, Vec<f64>) {
    let (pt, pt_1, var_pt) = pre_minimize(de_seasonalised_detrended_log_base_fee);

    let initial_position = vec![-3.928e-02, 2.873e-04, 4.617e-02, var_pt, var_pt, 0.2];
    let max_iterations = 2400;
    let solution = minimize(initial_position, &pt, &pt_1, max_iterations);

    let num_paths = 4000;
    let n_periods = 720;
    let (simulated_prices, params) = post_minimize(
        &solution,
        de_seasonalised_detrended_log_base_fee,
        n_periods,
        num_paths,
    )
    .unwrap();

    (simulated_prices, params)
}
