// ============================================================================
// RISC ZERO GUEST PROGRAM - PROOF COMPOSITION
// ============================================================================
// This program runs inside the RISC Zero ZK-VM to generate a zero-knowledge proof
// that verifies the correctness of gas fee option reserve price calculations.
//
// KEY CONCEPT: This is a "proof composition" - it verifies 7 sub-proofs and
// combines them into a single proof that can be efficiently verified on-chain.
// ============================================================================

// Import input/output structures for all computation stages
use core::{
    AddTwap7dErrorBoundFloatingInput,           // Input for 7-day TWAP verification
    CalculatePtPt1ErrorBoundFloatingInput,      // Input for Markov transition matrix verification
    HashingFeltOutput,                           // Output from data hashing
    ProofCompositionInput,                       // Combined input from all computations
    ProofCompositionOutput,                      // Final output committed to the proof
    RemoveSeasonalityErrorBoundFloatingInput,   // Input for time series decomposition verification
    SimulatePriceVerifyPositionInput,           // Input for Monte Carlo simulation verification
    TwapErrorBoundInput,                         // Input for TWAP verification
};

// Import RISC Zero ZK-VM runtime environment and serialization utilities
use risc0_zkvm::{guest::env, serde};

// Import guest program IDs for all sub-proofs
// Each ID uniquely identifies a guest program and is used to verify its receipts
use add_twap_7d_error_bound_floating_methods::ADD_TWAP_7D_ERROR_BOUND_FLOATING_GUEST_ID;
use remove_seasonality_error_bound_floating_methods::REMOVE_SEASONALITY_ERROR_BOUND_FLOATING_GUEST_ID;
use calculate_pt_pt1_error_bound_floating_methods::CALCULATE_PT_PT1_ERROR_BOUND_FLOATING_GUEST_ID;
use simulate_price_verify_position_floating_methods::SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ID;
use twap_error_bound_floating_methods::TWAP_ERROR_BOUND_FLOATING_GUEST_ID;
use max_return_floating_methods::MAX_RETURN_FLOATING_GUEST_ID;
use hashing_felts_methods::HASHING_FELTS_GUEST_ID;

// Import fixed-point arithmetic utilities for Starknet compatibility
// UFixedPoint123x128 represents numbers with 123 bits for the integer part and 128 bits for the fractional part
use guest_fixed_utils::{StorePacking, UFixedPoint123x128};

/// Helper function to convert a floating point value to a hex string representation
/// of a fixed-point packed number using the UFixedPoint123x128 type.
///
/// This conversion is necessary for on-chain verification on Starknet, which uses
/// fixed-point arithmetic instead of floating-point to ensure deterministic results.
///
/// Process:
/// 1. Convert f64 to UFixedPoint123x128 (123 bits integer, 128 bits fractional)
/// 2. Pack the fixed-point number into a compact representation
/// 3. Convert to hex string for easy serialization and on-chain storage
fn to_fixed_packed_hex(value: f64) -> String {
    UFixedPoint123x128::pack(UFixedPoint123x128::from(value)).to_hex_string()
}

fn main() {
    // ========== STEP 1: READ INPUT DATA ==========
    // Read the ProofCompositionInput from the host
    // This contains all the data and parameters needed to verify the sub-proofs
    let data: ProofCompositionInput = env::read();

    // ========== STEP 2: VERIFY SUB-PROOF #1 - DATA HASHING ==========
    // Verify that the hash of the 8-month gas fee data is correct
    // This ensures data integrity and creates a commitment that can be verified on-chain
    //
    // The env::verify() function checks that:
    // 1. A receipt for HASHING_FELTS_GUEST_ID exists in the assumptions
    // 2. The receipt's output matches the provided HashingFeltOutput
    //
    // If verification fails, the entire proof generation fails
    env::verify(
        HASHING_FELTS_GUEST_ID,  // Guest program ID for the hashing sub-proof
        &serde::to_vec(&HashingFeltOutput {
            hash: data.data_8_months_hash,           // The hash commitment
            f64_inputs: data.data_8_months.clone(),  // The original 8-month data
        })
        .unwrap(),
    )
    .unwrap();

    // ========== STEP 3: VERIFY SUB-PROOF #2 - MAXIMUM RETURN ==========
    // Verify that the maximum return (volatility measure) was calculated correctly
    // from the 8-month historical data
    //
    // Output format: (input_data, max_return_result)
    // This ensures the volatility calculation used for option pricing is correct
    env::verify(
        MAX_RETURN_FLOATING_GUEST_ID,  // Guest program ID for max return calculation
        &serde::to_vec(&(data.data_8_months.clone(), data.max_return)).unwrap(),
    )
    .unwrap();

    // ========== STEP 4: EXTRACT 3-MONTH DATA SUBSET ==========
    // Extract the subset needed for reserve price, TWAP, and time series decomposition
    //
    // DEVELOPER NOTE: Data Subset Configuration
    // =========================================
    // Production configuration: Extract last 2160 hours (90 days / 3 months) from 5760 hours (8 months)
    //   let data_3_months = data.data_8_months[data.data_8_months.len().saturating_sub(2160)..].to_vec();
    //
    // Current configuration: Extract last 720 hours (30 days / 1 month) from 1440 hours (2 months)
    // This aligns with the POC's reduced data availability.
    //
    // The subset is used for:
    //   - Reserve price calculation (requires sufficient data for time series decomposition)
    //   - TWAP calculation (time-weighted average price)
    //   - Seasonality and trend analysis
    //
    // IMPORTANT: When scaling to production (8 months total), update to extract 2160 hours:
    //   let data_3_months = data.data_8_months[data.data_8_months.len().saturating_sub(2160)..].to_vec();
    let data_3_months =
        data.data_8_months[data.data_8_months.len().saturating_sub(720)..].to_vec();

    // ========== STEP 5: VERIFY SUB-PROOF #3 - TWAP (TIME-WEIGHTED AVERAGE PRICE) ==========
    // Verify that the time-weighted average price was calculated correctly
    // TWAP is used as a reference price for gas fee options
    //
    // This checks that the TWAP calculation is within the specified tolerance (typically 1%)
    let twap_error_bound_input = TwapErrorBoundInput {
        avg_hourly_gas_fee: data_3_months.clone(),  // 90 days of hourly gas fees
        twap_tolerance: data.twap_tolerance,         // Acceptable deviation (1%)
        twap_result: data.twap_result,               // Expected TWAP result
    };

    env::verify(
        TWAP_ERROR_BOUND_FLOATING_GUEST_ID,  // Guest program ID for TWAP verification
        &serde::to_vec(&twap_error_bound_input).unwrap(),
    )
    .unwrap();

    // ========== STEP 6: VERIFY SUB-PROOF #4 - SEASONALITY REMOVAL ==========
    // Verify that the time series decomposition was performed correctly
    //
    // Time series decomposition separates gas fee data into three components:
    // 1. Trend: Long-term movement (captured by slope and intercept)
    // 2. Seasonality: Recurring patterns (hourly patterns captured by season_param)
    // 3. Residuals: Random fluctuations (de_seasonalised_detrended_log_base_fee)
    //
    // This decomposition is essential for accurate price modeling and forecasting
    let remove_seasonality_error_bound_input = RemoveSeasonalityErrorBoundFloatingInput {
        data: data_3_months.clone(),            // Original 90-day gas fee data
        slope: data.slope,                       // Linear trend slope
        intercept: data.intercept,               // Linear trend intercept
        // Residuals after removing trend and seasonality (in log space)
        de_seasonalised_detrended_log_base_fee: data.de_seasonalised_detrended_log_base_fee.clone(),
        season_param: data.season_param.clone(), // 24 hourly seasonal parameters
        tolerance: data.floating_point_tolerance, // Precision tolerance (0.00001%)
    };

    env::verify(
        REMOVE_SEASONALITY_ERROR_BOUND_FLOATING_GUEST_ID,  // Guest program ID
        &serde::to_vec(&remove_seasonality_error_bound_input).unwrap(),
    )
    .unwrap();

    // ========== STEP 7: VERIFY SUB-PROOF #5 - 7-DAY TWAP ==========
    // Verify that the 7-day rolling time-weighted average price was calculated correctly
    //
    // The 7-day TWAP serves as a mean reversion level in the price model
    // Gas prices tend to revert to this 7-day average, which is a key assumption
    // in the Markov chain model used for price simulation
    let add_twap_7d_error_bound_input = AddTwap7dErrorBoundFloatingInput {
        data: data_3_months.clone(),              // 90 days of hourly gas fees
        twap_7d: data.twap_7d.clone().clone(),   // 7-day rolling TWAP values
        tolerance: data.floating_point_tolerance, // Precision tolerance (0.00001%)
    };

    env::verify(
        ADD_TWAP_7D_ERROR_BOUND_FLOATING_GUEST_ID,  // Guest program ID
        &serde::to_vec(&add_twap_7d_error_bound_input).unwrap(),
    )
    .unwrap();

    // ========== STEP 8: VERIFY SUB-PROOF #6 - MARKOV CHAIN TRANSITION PROBABILITIES ==========
    // Verify that the transition probability matrices (pt and pt_1) were calculated correctly
    //
    // The Markov chain model uses these transition matrices to simulate future gas prices:
    // - pt: Probability of transitioning between price states in period t
    // - pt_1: Probability of transitioning between price states in period t-1
    //
    // These matrices are derived from the residuals (de_seasonalised_detrended_log_base_fee)
    // and capture the stochastic behavior of gas prices after removing trend and seasonality
    let calculate_pt_pt1_error_bound_input = CalculatePtPt1ErrorBoundFloatingInput {
        // Residuals used to estimate transition probabilities
        de_seasonalised_detrended_log_base_fee: data.de_seasonalised_detrended_log_base_fee.clone(),
        pt: data.pt.clone(),                      // Transition matrix for period t
        pt_1: data.pt_1.clone(),                  // Transition matrix for period t-1
        tolerance: data.floating_point_tolerance, // Precision tolerance (0.00001%)
    };

    env::verify(
        CALCULATE_PT_PT1_ERROR_BOUND_FLOATING_GUEST_ID,  // Guest program ID
        &serde::to_vec(&calculate_pt_pt1_error_bound_input).unwrap(),
    )
    .unwrap();

    // ========== STEP 9: VERIFY SUB-PROOF #7 - PRICE SIMULATION AND POSITION VERIFICATION ==========
    // This is the most computationally intensive verification step
    //
    // It verifies that:
    // 1. Monte Carlo price simulations were executed correctly (4000 paths)
    // 2. Gradient descent optimization converged to find the optimal reserve price
    // 3. Option positions (delta, vega, theta) are within acceptable risk bounds
    // 4. The reserve price satisfies all constraints
    //
    // This is the core of the reserve price calculation, ensuring that the price
    // is fair and that option sellers are adequately compensated for the risk
    let simulate_price_verify_position_input = SimulatePriceVerifyPositionInput {
        start_timestamp: data.start_timestamp,    // Start of analysis period
        end_timestamp: data.end_timestamp,        // End of analysis period
        data_length: data_3_months.len(),        // Number of data points (POC: 720, Production: 2160)
        positions: data.positions.clone(),        // Optimized option positions
        pt: data.pt.clone(),                      // Markov transition matrix t
        pt_1: data.pt_1.clone(),                  // Markov transition matrix t-1
        gradient_tolerance: data.gradient_tolerance, // Convergence tolerance (5%)
        // Residuals for stochastic price simulation
        de_seasonalised_detrended_log_base_fee: data.de_seasonalised_detrended_log_base_fee.clone(),
        n_periods: data.n_periods,                // Number of simulation periods (configurable)
        num_paths: data.num_paths,                // 4000 Monte Carlo simulation paths
        season_param: data.season_param.clone(),  // 24 hourly seasonal parameters
        twap_7d: data.twap_7d.clone(),           // 7-day TWAP (mean reversion level)
        slope: data.slope,                        // Linear trend slope
        intercept: data.intercept,                // Linear trend intercept
        reserve_price: data.reserve_price,        // Calculated reserve price
        tolerance: data.reserve_price_tolerance,  // Reserve price tolerance (5%)
    };

    env::verify(
        SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ID,  // Guest program ID
        &serde::to_vec(&simulate_price_verify_position_input).unwrap(),
    )
    .unwrap();

    // ========== STEP 10: CONSTRUCT OUTPUT FOR ON-CHAIN VERIFICATION ==========
    // After all 7 sub-proofs have been verified, construct the final output
    // that will be committed to the ZK proof and made available for on-chain verification
    //
    // All numerical values are converted to fixed-point hex strings for Starknet compatibility
    let output = ProofCompositionOutput {
        // Data commitment (hash of 8-month historical data)
        data_8_months_hash: data.data_8_months_hash,

        // Overall timestamp range for the proof
        start_timestamp: data.start_timestamp,
        end_timestamp: data.end_timestamp,

        // Reserve price calculation results (uses 3-month period)
        reserve_price_start_timestamp: data.start_timestamp,  // 90-day period start
        reserve_price_end_timestamp: data.end_timestamp,      // 90-day period end
        reserve_price: to_fixed_packed_hex(data.reserve_price), // Minimum option selling price

        // TWAP calculation results (uses 3-month period)
        twap_start_timestamp: data.start_timestamp,           // 90-day period start
        twap_end_timestamp: data.end_timestamp,               // 90-day period end
        twap_result: to_fixed_packed_hex(data.twap_result),  // Time-weighted average gas price

        // Maximum return (volatility) calculation (uses full 8-month period)
        max_return_start_timestamp: data.data_8_months_start_timestamp, // 240-day period start
        max_return_end_timestamp: data.data_8_months_end_timestamp,     // 240-day period end
        max_return: to_fixed_packed_hex(data.max_return),    // Historical volatility measure

        // Tolerances used for verification (converted to fixed-point for on-chain checks)
        floating_point_tolerance: to_fixed_packed_hex(data.floating_point_tolerance), // 0.00001%
        reserve_price_tolerance: to_fixed_packed_hex(data.reserve_price_tolerance),   // 5%
        gradient_tolerance: to_fixed_packed_hex(data.gradient_tolerance),             // 5%
        twap_tolerance: to_fixed_packed_hex(data.twap_tolerance),                     // 1%
    };

    // ========== STEP 11: COMMIT OUTPUT TO THE PROOF ==========
    // The env::commit() function writes the output to the ZK proof's public journal
    // This makes the output available to anyone verifying the proof on-chain
    //
    // The public journal contains:
    // - Data hash (to verify data integrity)
    // - Reserve price (the key result)
    // - TWAP (for reference pricing)
    // - Max return (for volatility assessment)
    // - All relevant timestamps and tolerances
    //
    // Anyone can verify this proof on-chain and trust these results without
    // re-executing the expensive computations
    env::commit(&output);
}
