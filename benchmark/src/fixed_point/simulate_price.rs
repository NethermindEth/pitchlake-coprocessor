use std::ops::Add;

use eyre::Result;
use nalgebra::{ComplexField, DMatrix, DVector};
use num_traits::Zero;
use rand::thread_rng;
use rand_distr::Distribution;
use statrs::distribution::{Binomial, Normal};

use crate::floating_point::Solution;
use approx::AbsDiffEq;

use super::{neg_log_likelihood, FixedPoint};

fn function_value(
    position: &[FixedPoint],
    pt: &DVector<FixedPoint>,
    pt_1: &DVector<FixedPoint>,
) -> FixedPoint {
    neg_log_likelihood(position, pt, pt_1)
}

fn gradient(
    position: &[FixedPoint],
    pt: &DVector<FixedPoint>,
    pt_1: &DVector<FixedPoint>,
) -> Vec<FixedPoint> {
    let mut x: Vec<_> = position.to_vec();
    let current = function_value(position, pt, pt_1);

    position
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, x_i)| {
            let h = if x_i.is_zero() {
                FixedPoint::default_epsilon() * FixedPoint::from_num(1.0e10)
            } else {
                (FixedPoint::default_epsilon() * x_i.abs()).sqrt()
            };

            // assert!(h.is_finite());

            x[i] = x_i + h;

            let forward = function_value(&x, pt, pt_1);

            x[i] = x_i;

            let d_i = (forward - current) / h;

            // assert!(d_i.is_finite());

            d_i
        })
        .collect()
}

fn is_saddle_point(gradient: &[FixedPoint], tolerance: FixedPoint) -> bool {
    gradient.iter().all(|dx| dx.abs() <= tolerance)
}

// ArmijoLineSearch::new(0.5, 1.0, 0.5)
fn search(
    initial_position: &[FixedPoint],
    direction: &[FixedPoint],
    pt: &DVector<FixedPoint>,
    pt_1: &DVector<FixedPoint>,
) -> Vec<FixedPoint> {
    let initial_value = function_value(initial_position, pt, pt_1);
    let gradient = gradient(initial_position, pt, pt_1);
    let control_parameter = FixedPoint::from_num(0.5);
    let initial_step_width = FixedPoint::from_num(1.0);
    let decay_factor = FixedPoint::from_num(0.5);

    let m = gradient
        .iter()
        .zip(direction)
        .map(|(g, d)| *g * *d)
        .fold(FixedPoint::zero(), Add::add);
    let t = -control_parameter * m;

    assert!(t > FixedPoint::zero());

    let mut step_width = initial_step_width;

    loop {
        let position: Vec<_> = initial_position
            .iter()
            .cloned()
            .zip(direction)
            .map(|(x, d)| x + step_width * *d)
            .collect();
        let value = function_value(&position, pt, pt_1);

        if value <= initial_value - step_width * t {
            return position;
        }

        step_width *= decay_factor;
    }
}

// error: currently using FixedPoint, we suffer from division by zero error
pub fn minimize(
    initial_position: Vec<FixedPoint>,
    pt: &DVector<FixedPoint>,
    pt_1: &DVector<FixedPoint>,
    max_iterations: u64,
) -> Solution<FixedPoint> {
    let mut position = initial_position;
    let mut value = function_value(&position, pt, pt_1);

    let gradient_tolerance = FixedPoint::from_num(1.0e-4);
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
    de_seasonalised_detrended_log_base_fee: &DVector<FixedPoint>,
) -> (DVector<FixedPoint>, DVector<FixedPoint>, FixedPoint) {
    // Prepare time series data
    let pt = DVector::from_row_slice(&de_seasonalised_detrended_log_base_fee.as_slice()[1..]);
    let pt_1 = DVector::from_row_slice(
        &de_seasonalised_detrended_log_base_fee.as_slice()
            [..de_seasonalised_detrended_log_base_fee.len() - 1],
    );

    // Perform the minimization
    let var_pt = pt.iter().map(|&x| x * x).fold(FixedPoint::zero(), Add::add)
        / FixedPoint::from_num(pt.len() as f64);

    (pt, pt_1, var_pt)
}

pub fn post_minimize(
    solution: &Solution<FixedPoint>,
    de_seasonalised_detrended_log_base_fee: &DVector<FixedPoint>,
    n_periods: usize,
    num_paths: usize,
) {
    //-> Result<(DMatrix<FixedPoint>, Vec<FixedPoint>)> {
    let dt = FixedPoint::from_num(1.0 / (365.0 * 24.0));

    // Extract the optimized parameters
    let params = &solution.position;

    // let alpha = params[0] / dt;
    // let kappa = (1.0 - params[1]) / dt;
    // let mu_j = params[2];
    // let sigma = (params[3] / dt).sqrt();
    // let sigma_j = params[4].sqrt();
    // let lambda_ = params[5] / dt;

    // // RNG for stochastic processes
    // let mut rng = thread_rng();

    // // Simulate the Poisson process (jumps)
    // let binom = Binomial::new(lambda_ * dt, 1)?;
    // let mut jumps = DMatrix::zeros(n_periods, num_paths);
    // for i in 0..n_periods {
    //     for j in 0..num_paths {
    //         jumps[(i, j)] = binom.sample(&mut rng) as f64;
    //     }
    // }

    // // Initialize simulated prices
    // let mut simulated_prices = DMatrix::zeros(n_periods, num_paths);
    // let initial_price =
    //     de_seasonalised_detrended_log_base_fee[de_seasonalised_detrended_log_base_fee.len() - 1];
    // for j in 0..num_paths {
    //     simulated_prices[(0, j)] = initial_price;
    // }

    // // Generate standard normal variables
    // let normal = Normal::new(0.0, 1.0).unwrap();
    // let mut n1 = DMatrix::zeros(n_periods, num_paths);
    // let mut n2 = DMatrix::zeros(n_periods, num_paths);
    // for i in 0..n_periods {
    //     for j in 0..num_paths {
    //         n1[(i, j)] = normal.sample(&mut rng);
    //         n2[(i, j)] = normal.sample(&mut rng);
    //     }
    // }
    // // Simulate prices over time
    // for i in 1..n_periods {
    //     for j in 0..num_paths {
    //         let prev_price = simulated_prices[(i - 1, j)];
    //         let current_n1 = n1[(i, j)];
    //         let current_n2 = n2[(i, j)];
    //         let current_j = jumps[(i, j)];

    //         simulated_prices[(i, j)] = alpha * dt
    //             + (1.0 - kappa * dt) * prev_price
    //             + sigma * dt.sqrt() * current_n1
    //             + current_j * (mu_j + sigma_j * current_n2);
    //     }
    // }

    // Ok((simulated_prices, params.to_vec()))
}
