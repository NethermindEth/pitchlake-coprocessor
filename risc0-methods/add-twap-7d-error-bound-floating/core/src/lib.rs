use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AddTwap7dErrorBoundFloatingInput {
    pub data: Vec<f64>,
    pub twap_7d: Vec<f64>,
    pub tolerance: f64,
}
