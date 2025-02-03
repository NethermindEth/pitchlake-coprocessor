use std::{f64::EPSILON, ops::Add};

use nalgebra::DVector;
use optimization::types::Solution;

use super::neg_log_likelihood;

fn function_value(position: &[f64], pt: &DVector<f64>, pt_1: &DVector<f64>) -> f64 {
    neg_log_likelihood(position, pt, pt_1)
}

fn gradient(position: &[f64], pt: &DVector<f64>, pt_1: &DVector<f64>) -> Vec<f64> {
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
) -> Solution {
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