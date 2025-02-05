use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReservePriceFloatingInput {
    pub period_start_timestamp: i64,
    pub period_end_timestamp: i64,
    pub season_param: DVector<f64>,
    pub de_seasonalized_detrended_simulated_prices: DMatrix<f64>,
    pub twap_7d: Vec<f64>,
    pub slope: f64,
    pub intercept: f64,
    pub log_base_fee_len: usize,
}
