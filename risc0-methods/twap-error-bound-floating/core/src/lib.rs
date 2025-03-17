use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TwapErrorBoundInput {
    pub avg_hourly_gas_fee: Vec<f64>,
    pub twap_tolerance: f64,
    pub twap_result: f64,
}
