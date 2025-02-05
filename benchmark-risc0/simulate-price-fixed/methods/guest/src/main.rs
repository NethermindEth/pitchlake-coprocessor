use benchmark::fixed_point::{minimize, post_minimize, pre_minimize, FixedPoint};
use nalgebra::DVector;
use risc0_zkvm::guest::env;

fn main() {
    let data: DVector<FixedPoint> = env::read();
    let (pt, pt_1, var_pt) = pre_minimize(&data);
    let max_iteration = 10;
    let solution = minimize(
        vec![
            FixedPoint::from_num(-3.928e-02),
            FixedPoint::from_num(2.873e-04),
            FixedPoint::from_num(4.617e-02),
            var_pt,
            var_pt,
            FixedPoint::from_num(0.2),
        ],
        &pt,
        &pt_1,
        max_iteration,
    );

    let num_paths = 4000;
    let n_periods = 720;
    let (simulated_prices, params) = post_minimize(&solution, &data, n_periods, num_paths).unwrap();
    // exposing data as public input so we can use it to assert output == this input in proof composition
    // yes it will add overhead
    // (DVector<f64>, DMatrix<f64>, Vec<f64>)
    env::commit(&(data, simulated_prices, params));
}

// pub fn pre_minimize(
//     de_seasonalised_detrended_log_base_fee: &DVector<f64>,
// ) -> (DVector<f64>, DVector<f64>, f64) { //  (pt, pt_1, var_pt)

// pub fn minimize(
//     initial_position: Vec<f64>, // vec![-3.928e-02, 2.873e-04, 4.617e-02, var_pt, var_pt, 0.2],
//     pt: &DVector<f64>,
//     pt_1: &DVector<f64>,
//     max_iterations: u64,
// ) -> Solution {

// pub fn post_minimize(
//     solution: &Solution,
//     de_seasonalised_detrended_log_base_fee: &DVector<f64>,
//     n_periods: usize,
//     num_paths: usize,
// ) -> Result<(DMatrix<f64>, Vec<f64>)> { // (simulated_prices, params)
