use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VolatilityInputs {
    pub base_fee_per_gases: Vec<Option<f64>>,
    pub ln_results: Vec<f64>,
}