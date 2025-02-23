use nalgebra::DVector;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RemoveSeasonalityErrorBoundFloatingInput {
    pub data: Vec<(i64, f64)>,
    pub slope: f64,
    pub intercept: f64,
    pub de_seasonalised_detrended_log_base_fee: DVector<f64>,
    pub season_param: DVector<f64>,
    pub tolerance: f64,
}