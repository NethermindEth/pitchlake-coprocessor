use nalgebra::DVector;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProofCompositionInput {
    pub data_8_months: Vec<(i64, f64)>,
    pub positions: Vec<f64>,
    pub pt: DVector<f64>,
    pub pt_1: DVector<f64>,
    pub gradient_tolerance: f64,
    pub de_seasonalised_detrended_log_base_fee: DVector<f64>,
    pub n_periods: usize,
    pub num_paths: usize,
    pub season_param: DVector<f64>,
    pub twap_7d: Vec<f64>,
    pub slope: f64,
    pub intercept: f64,
    pub reserve_price: f64,
    pub floating_point_tolerance: f64,
    pub reserve_price_tolerance: f64,
    pub twap_tolerance: f64,
    pub twap_result: f64,
    pub max_return: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProofCompositionOutput {
    pub data_8_months: Vec<(i64, f64)>,
    pub reserve_price: f64,
    pub floating_point_tolerance: f64,
    pub reserve_price_tolerance: f64,
    pub twap_tolerance: f64,
    pub twap_result: f64,
    pub max_return: f64,
}
