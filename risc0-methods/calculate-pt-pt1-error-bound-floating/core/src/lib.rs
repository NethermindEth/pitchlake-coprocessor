use nalgebra::DVector;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CalculatePtPt1ErrorBoundFloatingInput {
    pub de_seasonalised_detrended_log_base_fee: DVector<f64>,
    pub pt: DVector<f64>,
    pub pt_1: DVector<f64>,
    pub tolerance: f64,
}
