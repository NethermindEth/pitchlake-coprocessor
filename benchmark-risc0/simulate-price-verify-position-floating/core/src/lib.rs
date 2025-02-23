use nalgebra::DVector;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SimulatePriceVerifyPositionInput {
    pub data: Vec<(i64, f64)>,
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
    pub tolerance: f64,
}
