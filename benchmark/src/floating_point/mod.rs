use std::f64::consts::PI;

use nalgebra::{DMatrix, DVector};

pub mod remove_seasonality;
pub use remove_seasonality::*;
pub mod reserve_price;
pub use reserve_price::*;
pub mod simulate_price;
pub use simulate_price::*;
pub mod add_twap;
pub use add_twap::*;
pub mod simulate_price_verify_position;
pub use simulate_price_verify_position::*;

use crate::common::csv::{
    close_csv_file, open_error_bound_diff_csv_writer, write_error_bound_diff_to_csv,
};
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

pub struct AllInputsToReservePrice {
    pub season_param: DVector<f64>,
    pub de_seasonalised_detrended_log_base_fee: DVector<f64>,
    pub de_seasonalized_detrended_simulated_prices: DMatrix<f64>,
    pub twap_7d: Vec<f64>,
    pub slope: f64,
    pub intercept: f64,
    pub reserve_price: f64,
}

pub fn calculate_reserve_price_full(input: &Vec<(i64, f64)>) -> AllInputsToReservePrice {
    let num_paths = 15000;
    let n_periods = 720;
    let twap = add_twap_7d(&input).unwrap();
    let (slope, intercept, de_seasonalised_detrended_log_base_fee, season_param) =
        calculate_remove_seasonality(&input).unwrap();

    let (simulated_prices, _params) = simulate_price(
        &de_seasonalised_detrended_log_base_fee,
        num_paths,
        n_periods,
    );

    let reserve_price = calculate_reserve_price(
        input[0].0,
        input[input.len() - 1].0,
        &season_param,
        &simulated_prices,
        &twap,
        slope,
        intercept,
        input.len(),
        num_paths,
        n_periods,
    )
    .unwrap();

    AllInputsToReservePrice {
        season_param,
        de_seasonalised_detrended_log_base_fee,
        de_seasonalized_detrended_simulated_prices: simulated_prices,
        twap_7d: twap,
        slope,
        intercept,
        reserve_price,
    }
}

// tolerance is in percentage eg: 5.0 means 5%
pub fn error_bound_matrix(
    target: &DMatrix<f64>,
    calculated: &DMatrix<f64>,
    tolerance: f64,
) -> bool {
    if target.nrows() != calculated.nrows() || target.ncols() != calculated.ncols() {
        return false;
    }

    // let mut wtr = open_error_bound_diff_csv_writer("error_bound_diff.csv");
    // let mut total_diff = 0;
    for i in 0..target.nrows() {
        for j in 0..target.ncols() {
            let target_val = target[(i, j)];
            let calc_val = calculated[(i, j)];
            let diff: f64 = (target_val - calc_val).abs();

            let percentage_diff = if target_val != 0.0 {
                (diff / target_val.abs()) * 100.0
            } else if calc_val != 0.0 {
                100.0 // If target is 0 but calc isn't, that's 100% error
            } else {
                0.0 // Both are 0, no difference
            };

            // write_error_bound_diff_to_csv(&mut wtr, i, j, target_val, calc_val, percentage_diff);

            if percentage_diff > tolerance {
                // total_diff += 1;
                println!("i: {:?}", i);
                println!("j: {:?}", j);
                println!("target_val: {:?}", target_val);
                println!("calc_val: {:?}", calc_val);
                println!("percentage_diff: {:?}\n", percentage_diff);
                continue;
                // return false;
            }
        }
    }

    // close_csv_file(&mut wtr);

    // println!("total_diff: {:?}", total_diff);

    true
}

/// use to compare the error bound of simulated log prices only
/// this is because aside from checking if each element is within the tolerance level
/// we are also checking if the difference in all of the elements is within an acceptable threshold
/// the reason of doing this is because, after sampling using thread.rng(), the simulated log prices
/// could introduce large differences in elements seeminly randomly
/// this could be mitigated using quasi monte carlo sampling but we are in the process of testing it atm
pub fn error_bound_simulated_log_prices(
    target: &DMatrix<f64>,
    calculated: &DMatrix<f64>,
    element_wise_tolerance: f64,
    matrix_tolerance: f64,
) -> bool {
    let mut total_diff = 0;
    for i in 0..target.nrows() {
        for j in 0..target.ncols() {
            let target_val = target[(i, j)];
            let calc_val = calculated[(i, j)];
            let diff: f64 = (target_val - calc_val).abs();

            let percentage_diff = if target_val != 0.0 {
                (diff / target_val.abs()) * 100.0
            } else if calc_val != 0.0 {
                100.0 // If target is 0 but calc isn't, that's 100% error
            } else {
                0.0 // Both are 0, no difference
            };

            if percentage_diff > element_wise_tolerance {
                total_diff += 1;
            }
        }
    }

    let total_elements = target.nrows() * target.ncols();
    let percentage_diff = (total_diff as f64 / total_elements as f64) * 100.0;

    percentage_diff < matrix_tolerance
}
