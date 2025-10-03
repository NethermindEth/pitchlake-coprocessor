// RISC0 PROOF GENERATION - CORE INPUT/OUTPUT STRUCTURES
//
// This module defines the data structures used for RISC0 zero-knowledge proof generation
// in the gas fee option pricing system. These structures are shared between the host
// (proof orchestration) and guest (RISC0 VM execution) programs.
//
// DATA VOLUME CONFIGURATION:
// ==========================
// The system has two main configuration modes:
//
// 1. PRODUCTION (Full Historical Data):
//    - Total data: 5760 hours (8 months of hourly fee data)
//    - Subset for reserve price: 2160 hours (3 months / 90 days)
//    - Use case: Production-grade reserve price calculation with full statistical confidence
//
// 2. POC (Reduced Data for Testing):
//    - Total data: 1440 hours (2 months of hourly fee data)
//    - Subset for reserve price: 720 hours (1 month / 30 days)
//    - Use case: Proof-of-concept and testing with limited onchain data availability
//
// CONFIGURATION COORDINATION:
// ===========================
// When changing data volumes, update ALL of the following:
// 1. This file: data_length field documentation (line 126)
// 2. Hashing guest: methods/hashing-felts-methods/guest/src/main.rs (assertion on line 21)
// 3. Proof composition guest: methods/proof-composition-.../guest/src/main.rs (line 109)
// 4. Message handler: proving-service/.../proof_composition/mod.rs (REQUIRED_HOURS constant)
// 5. Cairo contract: starknet-contracts/fossil-hash-store/src/lib.cairo (num_in_a_batch)
//
// PROOF COMPOSITION WORKFLOW:
// ===========================
// The ProofCompositionInput structure contains all data needed to verify 7 sub-proofs:
// 1. Data Hashing - Verifies integrity of historical fee data
// 2. Max Return - Calculates volatility from full historical data
// 3. TWAP - Time-weighted average price from subset
// 4. Seasonality Removal - Time series decomposition
// 5. 7-day TWAP - Mean reversion level calculation
// 6. Markov Transition Probabilities - Stochastic model parameters
// 7. Price Simulation - Monte Carlo simulation and reserve price validation

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
    pub data_8_months_start_timestamp: i64,
    pub data_8_months_end_timestamp: i64,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    // Specific timestamp ranges for each calculation type
    pub twap_start_timestamp: i64,
    pub twap_end_timestamp: i64,
    pub reserve_price_start_timestamp: i64,
    pub reserve_price_end_timestamp: i64,
    pub max_return_start_timestamp: i64,
    pub max_return_end_timestamp: i64,
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
    pub reserve_price_start_timestamp: i64,
    pub reserve_price_end_timestamp: i64,
    pub reserve_price: String,
    pub twap_start_timestamp: i64,
    pub twap_end_timestamp: i64,
    pub twap_result: String,
    pub max_return_start_timestamp: i64,
    pub max_return_end_timestamp: i64,
    pub max_return: String,
    pub floating_point_tolerance: String,
    pub reserve_price_tolerance: String,
    pub twap_tolerance: String,
    pub gradient_tolerance: String,
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
    // DEVELOPER NOTE: Data Length Configuration
    // =========================================
    // Production: 2160 hours (90 days / 3 months of hourly data)
    // Current POC: 720 hours (30 days / 1 month of hourly data)
    //
    // This field specifies the length of the data subset used for reserve price
    // calculation. It must match the actual length of the data provided to the
    // simulation and should align with the proof composition guest program's
    // data extraction logic.
    pub data_length: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TwapErrorBoundInput {
    pub avg_hourly_gas_fee: Vec<f64>,
    pub twap_tolerance: f64,
    pub twap_result: f64,
}
