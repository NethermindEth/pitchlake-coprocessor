use nalgebra::DVector;
use serde::{Deserialize, Serialize};
use starknet_core::types::Felt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AddTwap7dErrorBoundFloatingInput {
    pub data: Vec<f64>,
    pub twap_7d: Vec<f64>,
    pub tolerance: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CalculatePtPt1ErrorBoundFloatingInput {
    pub de_seasonalised_detrended_log_base_fee: DVector<f64>,
    pub pt: DVector<f64>,
    pub pt_1: DVector<f64>,
    pub tolerance: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HashingFeltInput {
    pub inputs: Vec<Felt>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HashingFeltOutput {
    pub hash: [u32; 8],
    pub f64_inputs: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MaxReturnInput {
    pub data: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProofCompositionInput {
    pub data_8_months: Vec<f64>,
    pub data_8_months_hash: [u32; 8],
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub positions: Vec<f64>,
    pub pt: DVector<f64>,
    pub pt_1: DVector<f64>,
    pub gradient_tolerance: f64,
    pub de_seasonalised_detrended_log_base_fee: DVector<f64>,
    pub n_periods: usize,
    pub num_paths: usize,
    pub season_param: DVector<f64>,
    pub twap_7d: Vec<f64>,
    pub slope: f64,
    pub intercept: f64,
    pub reserve_price: f64,
    pub floating_point_tolerance: f64,
    pub reserve_price_tolerance: f64,
    pub twap_tolerance: f64,
    pub twap_result: f64,
    pub max_return: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProofCompositionOutput {
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
pub struct RemoveSeasonalityErrorBoundFloatingInput {
    pub data: Vec<f64>,
    pub slope: f64,
    pub intercept: f64,
    pub de_seasonalised_detrended_log_base_fee: DVector<f64>,
    pub season_param: DVector<f64>,
    pub tolerance: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SimulatePriceVerifyPositionInput {
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub positions: Vec<f64>,
    pub pt: DVector<f64>,
    pub pt_1: DVector<f64>,
    pub gradient_tolerance: f64,
    pub de_seasonalised_detrended_log_base_fee: DVector<f64>,
    pub n_periods: usize,
    pub num_paths: usize,
    pub season_param: DVector<f64>,
    pub twap_7d: Vec<f64>,
    pub slope: f64,
    pub intercept: f64,
    pub reserve_price: f64,
    pub tolerance: f64,
    pub data_length: usize, // should always be 2160 . TODO: can remove
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TwapErrorBoundInput {
    pub avg_hourly_gas_fee: Vec<f64>,
    pub twap_tolerance: f64,
    pub twap_result: f64,
}
