use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

// TODO: considering passnig in as reference for Dvector and Dmatrix
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CalculateSimulatedLogPricesInput {
    pub period_start_timestamp: i64,
    pub period_end_timestamp: i64,
    pub season_param: DVector<f64>,
    pub de_seasonalized_detrended_simulated_prices: DMatrix<f64>,
    pub twap_7d: Vec<f64>,
    pub slope: f64,
    pub intercept: f64,
    pub log_base_fee_len: usize,
    pub num_paths: usize,
    pub n_periods: usize,
    pub supposed_simulated_log_prices: DMatrix<f64>,
    pub error_tolerance_in_percentage: f64,
    pub matrix_tolerance_in_percentage: f64,
}
