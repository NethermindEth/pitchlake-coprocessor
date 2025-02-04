use std::f64::consts::PI;

use nalgebra::{DMatrix, DVector};

pub mod remove_seasonality;
pub use remove_seasonality::*;
pub mod reserve_price;
pub use reserve_price::*;
pub mod simulate_price;
pub use simulate_price::*;
pub mod solution;
pub use solution::*;
pub mod add_twap;
pub use add_twap::*;

pub fn mrjpdf(params: &[f64], pt: &DVector<f64>, pt_1: &DVector<f64>) -> DVector<f64> {
    let (a, phi, mu_j, sigma_sq, sigma_sq_j, lambda) = (
        params[0], params[1], params[2], params[3], params[4], params[5],
    );
    let diff1 = pt
        - (DVector::from_element(pt.len(), a) + phi * pt_1 + DVector::from_element(pt.len(), mu_j));
    let diff2 = pt - (DVector::from_element(pt.len(), a) + phi * pt_1);

    let term1 = lambda
        * (-diff1.map(|x| x.powi(2)) / (2.0 * (sigma_sq + sigma_sq_j))).map(f64::exp)
        / ((2.0 * std::f64::consts::PI * (sigma_sq + sigma_sq_j)).sqrt());

    let term2 = (1.0 - lambda) * (-diff2.map(|x| x.powi(2)) / (2.0 * sigma_sq)).map(f64::exp)
        / ((2.0 * std::f64::consts::PI * sigma_sq).sqrt());

    term1 + term2
}

pub fn neg_log_likelihood(params: &[f64], pt: &DVector<f64>, pt_1: &DVector<f64>) -> f64 {
    let pdf_vals = mrjpdf(params, pt, pt_1);
    -(pdf_vals.map(|x| x + 1e-10).map(f64::ln).sum())
}

pub fn season_matrix(t: DVector<f64>) -> DMatrix<f64> {
    let n = t.len();
    let mut result = DMatrix::zeros(n, 12);

    for i in 0..n {
        let time = t[i];
        result[(i, 0)] = (2.0 * PI * time / 24.0).sin();
        result[(i, 1)] = (2.0 * PI * time / 24.0).cos();
        result[(i, 2)] = (4.0 * PI * time / 24.0).sin();
        result[(i, 3)] = (4.0 * PI * time / 24.0).cos();
        result[(i, 4)] = (8.0 * PI * time / 24.0).sin();
        result[(i, 5)] = (8.0 * PI * time / 24.0).cos();
        result[(i, 6)] = (2.0 * PI * time / (24.0 * 7.0)).sin();
        result[(i, 7)] = (2.0 * PI * time / (24.0 * 7.0)).cos();
        result[(i, 8)] = (4.0 * PI * time / (24.0 * 7.0)).sin();
        result[(i, 9)] = (4.0 * PI * time / (24.0 * 7.0)).cos();
        result[(i, 10)] = (8.0 * PI * time / (24.0 * 7.0)).sin();
        result[(i, 11)] = (8.0 * PI * time / (24.0 * 7.0)).cos();
    }

    result
}
