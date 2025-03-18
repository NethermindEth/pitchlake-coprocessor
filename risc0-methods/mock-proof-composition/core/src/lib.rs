use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProofCompositionOutputFixed {
    pub data_8_months_hash: [u32; 8],
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub reserve_price: String,
    pub floating_point_tolerance: String,
    pub reserve_price_tolerance: String,
    pub twap_tolerance: String,
    pub gradient_tolerance: String,
    pub twap_result: String,
    pub max_return: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProofCompositionOutput {
    pub data_8_months_hash: [u32; 8],
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub reserve_price: f64,
    pub floating_point_tolerance: f64,
    pub reserve_price_tolerance: f64,
    pub twap_tolerance: f64,
    pub gradient_tolerance: f64,
    pub twap_result: f64,
    pub max_return: f64,
}
