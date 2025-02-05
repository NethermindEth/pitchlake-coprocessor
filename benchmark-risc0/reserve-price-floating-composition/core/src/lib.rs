use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReservePriceFloatingCompositionInput {
    pub inputs: Vec<(i64, f64)>,
    pub season_param: DVector<f64>,
    pub de_seasonalised_detrended_log_base_fee: DVector<f64>,
    pub de_seasonalized_detrended_simulated_prices: DMatrix<f64>,
    pub twap_7d: Vec<f64>,
    pub slope: f64,
    pub intercept: f64,
    pub reserve_price: f64,
}
