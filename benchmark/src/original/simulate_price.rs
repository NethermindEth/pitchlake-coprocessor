use std::f64::EPSILON;
use std::ops::Add;

use eyre::{anyhow as err, Result};
use ndarray::prelude::*;
use ndarray::{stack, Array1, Array2, Axis};
use ndarray_linalg::LeastSquaresSvd;
use ndarray_rand::rand_distr::Normal;
use rand::prelude::*;
use rand_distr::Distribution;
use statrs::distribution::Binomial;

use crate::floating_point::Solution;
use crate::is_saddle_point;

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

fn neg_log_likelihood(params: &[f64], pt: &Array1<f64>, pt_1: &Array1<f64>) -> f64 {
    let pdf_vals = mrjpdf(params, pt, pt_1);
    -pdf_vals.mapv(|x| (x + 1e-10).ln()).sum()
}

pub fn function_value(position: &[f64], pt: &Array1<f64>, pt_1: &Array1<f64>) -> f64 {
    neg_log_likelihood(position, pt, pt_1)
}

pub fn gradient(position: &[f64], pt: &Array1<f64>, pt_1: &Array1<f64>) -> Vec<f64> {
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

// ArmijoLineSearch::new(0.5, 1.0, 0.5)
fn search(
    initial_position: &[f64],
    direction: &[f64],
    pt: &Array1<f64>,
    pt_1: &Array1<f64>,
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
    pt: &Array1<f64>,
    pt_1: &Array1<f64>,
    max_iterations: u64,
) -> (Solution<f64>, bool) {
    let mut position = initial_position;
    let mut value = function_value(&position, pt, pt_1);

    let gradient_tolerance = 1.0e-4;
    let mut iteration = 0;
    loop {
        let gradient = gradient(&position, pt, pt_1);

        if is_saddle_point(&gradient, gradient_tolerance) {
            return (Solution::new(position, value), true);
        }

        let direction: Vec<_> = gradient.into_iter().map(|g| -g).collect();
        let iter_xs = search(&position, &direction, pt, pt_1);
        position = iter_xs;
        value = function_value(&position, pt, pt_1);
        iteration += 1;

        if iteration == max_iterations {
            return (Solution::new(position, value), false);
        }
    }
}

fn simulate_prices(
    de_seasonalised_detrended_log_base_fee: ArrayView1<f64>,
    n_periods: usize,
    num_paths: usize,
    max_iterations: u64,
) -> Result<(Array2<f64>, Vec<f64>)> {
    let dt = 1.0 / (365.0 * 24.0);
    let pt = de_seasonalised_detrended_log_base_fee
        .slice(s![1..])
        .to_owned();
    let pt_1 = de_seasonalised_detrended_log_base_fee
        .slice(s![..-1])
        .to_owned();

    let var_pt = pt.var(0.0);

    let (solution, _is_saddle_point) = minimize(
        vec![-3.928e-02, 2.873e-04, 4.617e-02, var_pt, var_pt, 0.2],
        &pt,
        &pt_1,
        max_iterations,
    );

    let params = &solution.position;
    let alpha = params[0] / dt;
    let kappa = (1.0 - params[1]) / dt;
    let mu_j = params[2];
    let sigma = (params[3] / dt).sqrt();
    let sigma_j = params[4].sqrt();
    let lambda_ = params[5] / dt;

    let mut rng = thread_rng();
    let j: Array2<f64> = {
        let binom = Binomial::new(lambda_ * dt, 1)?;
        Array2::from_shape_fn((n_periods, num_paths), |_| binom.sample(&mut rng) as f64)
    };

    let mut simulated_prices = Array2::zeros((n_periods, num_paths));
    simulated_prices
        .slice_mut(s![0, ..])
        .assign(&Array1::from_elem(
            num_paths,
            de_seasonalised_detrended_log_base_fee
                [de_seasonalised_detrended_log_base_fee.len() - 1],
        ));

    let normal = Normal::new(0.0, 1.0).unwrap();
    let n1 = Array2::from_shape_fn((n_periods, num_paths), |_| normal.sample(&mut rng));
    let n2 = Array2::from_shape_fn((n_periods, num_paths), |_| normal.sample(&mut rng));

    for i in 1..n_periods {
        let prev_prices = simulated_prices.slice(s![i - 1, ..]);
        let current_n1 = n1.slice(s![i, ..]);
        let current_n2 = n2.slice(s![i, ..]);
        let current_j = j.slice(s![i, ..]);

        let new_prices = &(alpha * dt
            + (1.0 - kappa * dt) * &prev_prices
            + sigma * dt.sqrt() * &current_n1
            + &current_j * (mu_j + sigma_j * &current_n2));

        simulated_prices
            .slice_mut(s![i, ..])
            .assign(&new_prices.clone());
    }

    Ok((simulated_prices, params.to_vec()))
}
