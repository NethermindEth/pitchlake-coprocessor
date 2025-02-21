use nalgebra::DMatrix;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CalculateReservePriceFromSimulatedLogPricesInput {
    pub simulated_log_prices: DMatrix<f64>,
    pub twap_7d: Vec<f64>,
    pub n_periods: usize,
}
