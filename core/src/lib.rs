use fixed::types::I48F16;
use serde::{Deserialize, Serialize};
use simba::scalar::FixedI48F16;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VolatilityInputs {
    pub base_fee_per_gases: Vec<Option<f64>>,
    pub ln_results: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VolatilityInputsFixedPoint {
    pub base_fee_per_gases: Vec<Option<I48F16>>,
    pub ln_results: Vec<I48F16>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VolatilityInputsFixedPointSimba {
    pub base_fee_per_gases: Vec<Option<FixedI48F16>>,
    pub ln_results: Vec<FixedI48F16>,
}

