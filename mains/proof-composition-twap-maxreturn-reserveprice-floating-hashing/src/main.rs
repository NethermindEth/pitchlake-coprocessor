// Import function to add 7-day time-weighted average price (TWAP) with error bounds
use add_twap_7d_error_bound_floating::add_twap_7d_error_bound;

// Import common utilities for data processing and manipulation
use common::{
    // Functions to read CSV data and convert timestamps to dates
    common::dataframe::{read_data_from_file, replace_timestamp_with_date},
    // Floating-point arithmetic utilities for gas fee calculations
    floating_point,
    // Original algorithm implementations and array conversion utilities
    original::{self, convert_array1_to_dvec},
    // Mock data utilities for testing with 5760 average base fees (240 days * 24 hours)
    tests::mock::{convert_data_to_vec_of_tuples, get_5760_avg_base_fees_felt},
};

// Import core input structures for various computation stages
use core::{
    AddTwap7dErrorBoundFloatingInput, // Input for 7-day TWAP error bound calculation
    CalculatePtPt1ErrorBoundFloatingInput, // Input for price transition probability calculations
    HashingFeltInput,                 // Input for Starknet field element hashing
    MaxReturnInput,                   // Input for maximum return calculation
    ProofCompositionInput,            // Main input structure combining all computations
    RemoveSeasonalityErrorBoundFloatingInput, // Input for deseasonalizing time series data
    SimulatePriceVerifyPositionInput, // Input for Monte Carlo price simulation
    TwapErrorBoundInput,              // Input for TWAP verification with tolerance
};

// Import maximum return calculation function (analyzes historical gas fee volatility)
use max_return_floating::max_return;

// Import the RISC Zero guest program ELF binary and ID for ZK proof generation
use proof_composition_twap_maxreturn_reserveprice_floating_hashing_methods::{
    PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_HASHING_GUEST_ELF,
    PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_HASHING_GUEST_ID,
};

// Import seasonality removal function (detrends and deseasonalizes gas fee data)
use remove_seasonality_error_bound_floating::remove_seasonality_error_bound;

// Import RISC Zero ZK-VM utilities for proof generation
use risc0_zkvm::{default_prover, ExecutorEnv};
use std::thread;
use std::time::Duration;

// Import price simulation function that verifies option positions are within bounds
use simulate_price_verify_position_floating::simulate_price_verify_position as simulate_price_verify_position_receipt;

// Import TWAP calculation with error bounds for gas fee averaging
use twap_error_bound_floating::calculate_twap;

// Import transition probability calculation for Markov chain price model
use calculate_pt_pt1_error_bound_floating::calculate_pt_pt1_error_bound_floating;

// Import Starknet field element hashing (Pedersen/Poseidon hash for on-chain verification)
use hashing_felts::hash_felts;

fn main() {
    // ========== STEP 1: HASH GAS FEE DATA ==========
    // Get 5760 hourly average base fees (8 months of data: 240 days * 24 hours)
    // These represent Ethereum gas fees as Starknet field elements (felts)
    let inputs_felt = get_5760_avg_base_fees_felt();

    // Hash the gas fee data using Starknet-compatible hashing (Pedersen/Poseidon)
    // This creates a commitment to the data that can be verified on-chain
    // The hashing will eventually be performed on-chain for trustless verification
    let (hashing_receipt, hashing_res) = hash_felts(HashingFeltInput {
        inputs: inputs_felt,
    });

    // ========== STEP 2: CALCULATE MAXIMUM RETURN ==========
    // Extract the f64 (floating-point) representation of the gas fee data
    // This converts the Starknet field elements back to standard floating-point numbers
    let data_8_months = hashing_res.f64_inputs;

    // Calculate the maximum return (volatility measure) from 8 months of historical data
    // This analyzes the largest price swings and is used for option pricing parameters
    let input = MaxReturnInput {
        data: data_8_months.clone(),
    };
    let (max_return_receipt, max_return_res) = max_return(input);

    // ========== STEP 3: CALCULATE TIME-WEIGHTED AVERAGE PRICE (TWAP) ==========
    // TWAP compares the time-weighted average using:
    //   1. Actual block-by-block data (host computation)
    //   2. Hourly averaged gas fees (ZK-VM computation)
    // This ensures the ZK proof uses a reasonable approximation of the true average

    // Read historical gas fee data from CSV file (not used in this mock version)
    let df = read_data_from_file("data.csv");
    let _df = replace_timestamp_with_date(df).unwrap();

    // Extract the last 2160 hours (90 days) of data from the 8-month dataset
    // This represents the 3-month period used for reserve price calculation
    // 2160 = 90 days * 24 hours/day
    let data = data_8_months[data_8_months.len().saturating_sub(2160)..].to_vec();

    // NOTE: The code below shows how TWAP would be calculated from raw block data
    // This is commented out because we're using pre-aggregated hourly averages
    // let raw_data_period = filter_dataframe_by_date_range(df, data[0].0, data[data.len() - 1].0);
    // let raw_data: Vec<(i64, i64)> = convert_to_timestamp_base_fee_int_tuple(raw_data_period);
    // let twap_original = original::calculate_twap::calculate_twap(&raw_data);

    // Calculate TWAP using the floating-point algorithm on hourly averaged data
    // This is the "ground truth" that will be verified in the ZK proof
    let twap_original = floating_point::calculate_twap(&data);
    println!("twap_original: {:?}", twap_original);

    // Prepare input for TWAP verification with error bounds
    // The ZK proof will verify that the calculated TWAP is within tolerance (1.0%)
    let input = TwapErrorBoundInput {
        avg_hourly_gas_fee: data.clone(), // 90 days of hourly gas fees
        twap_tolerance: 1.0,              // 1% tolerance for TWAP calculation
        twap_result: twap_original,       // Expected TWAP result to verify against
    };

    // Generate a ZK proof receipt that TWAP was calculated correctly
    let (calculate_twap_receipt, _calculate_twap_res) = calculate_twap(input);

    // ========== STEP 4: CALCULATE RESERVE PRICE (HOST COMPUTATION) ==========
    // The reserve price is the minimum price at which gas fee options can be sold
    // This is computed on the host (not in ZK) to ensure numerical convergence
    // The ZK proof will verify the calculation is correct within tolerances

    // Define the time period for reserve price calculation
    let start_timestamp = 1708833600; // February 25, 2024, 00:00:00 UTC
                                      // Calculate end timestamp: 90 days later (3 months)
    let end_timestamp = 1708833600 + (3600 * 24 * 30 * 3);

    // n_periods: number of 3-hour periods in 90 days (720 = 90 days * 24 hours / 3 hours)
    // This is used for Markov chain state transitions in the price model
    let n_periods = 720;

    // Convert hourly gas fee data to (timestamp, fee) tuples starting from start_timestamp
    let data_with_timestamps = convert_data_to_vec_of_tuples(data.clone(), start_timestamp);

    // Calculate reserve price using the original algorithm on the host
    // Parameters: data, strike price (15000 gwei), and number of periods (720)
    // This performs time series decomposition, Monte Carlo simulation, and gradient descent
    let res = original::calculate_reserve_price(&data_with_timestamps, 15000, n_periods);

    // ========== STEP 5: DEFINE TOLERANCES FOR ZK VERIFICATION ==========
    // These tolerances determine how precisely the ZK proof must match the host computation

    let num_paths = 4000; // Number of Monte Carlo simulation paths
    let gradient_tolerance = 5e-2; // 5% tolerance for gradient descent convergence
    let floating_point_tolerance = 0.00001; // 0.00001% tolerance for floating-point arithmetic
    let reserve_price_tolerance = 5.0; // 5% tolerance for final reserve price

    // ========== STEP 6: VERIFY SEASONALITY REMOVAL ==========
    // This generates a ZK proof that the time series decomposition was performed correctly
    // Time series decomposition separates the data into:
    //   - Trend component (captured by slope and intercept)
    //   - Seasonal component (captured by season_param for each hour of the day)
    //   - Residual component (de_seasonalised_detrended_log_base_fee)
    let (remove_seasonality_error_bound_receipt, _remove_seasonality_error_bound_res) =
        remove_seasonality_error_bound(RemoveSeasonalityErrorBoundFloatingInput {
            data: data.clone(),       // Original 90-day gas fee data
            slope: res.slope,         // Linear trend slope
            intercept: res.intercept, // Linear trend intercept
            // The residuals after removing trend and seasonality (in log space)
            de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
                res.de_seasonalised_detrended_log_base_fee.clone(),
            ),
            // Seasonal parameters (24 values, one for each hour of the day)
            season_param: convert_array1_to_dvec(res.season_param.clone()),
            tolerance: floating_point_tolerance, // 0.00001% tolerance
        });

    // ========== STEP 7: VERIFY 7-DAY TWAP CALCULATION ==========
    // This generates a ZK proof that the 7-day rolling TWAP was calculated correctly
    // The 7-day TWAP is used as a reversion level in the mean-reverting price model
    let (add_twap_7d_error_bound_receipt, _add_twap_7d_error_bound_res) =
        add_twap_7d_error_bound(AddTwap7dErrorBoundFloatingInput {
            data: data.clone(),                  // Original 90-day gas fee data
            twap_7d: res.twap_7d.clone(),        // 7-day TWAP values calculated by host
            tolerance: floating_point_tolerance, // 0.00001% tolerance
        });

    // ========== STEP 8: VERIFY MARKOV CHAIN TRANSITION PROBABILITIES ==========
    // This generates a ZK proof that the transition probability matrices were calculated correctly
    // The Markov chain model uses two transition matrices:
    //   - pt: probability of moving between price states in period t
    //   - pt_1: probability of moving between price states in period t-1
    // These matrices are derived from the de_seasonalised_detrended_log_base_fee residuals
    let (calculate_pt_pt1_error_bound_receipt, _calculate_pt_pt1_error_bound_res) =
        calculate_pt_pt1_error_bound_floating(CalculatePtPt1ErrorBoundFloatingInput {
            // The residuals used to estimate the transition probabilities
            de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
                res.de_seasonalised_detrended_log_base_fee.clone(),
            ),
            pt: convert_array1_to_dvec(res.pt.clone()), // Transition matrix for period t
            pt_1: convert_array1_to_dvec(res.pt_1.clone()), // Transition matrix for period t-1
            tolerance: floating_point_tolerance,        // 0.00001% tolerance
        });

    // ========== STEP 9: SIMULATE PRICES AND VERIFY OPTION POSITIONS ==========
    // This is the most computationally intensive step, generating a ZK proof that:
    //   1. Monte Carlo price simulations were run correctly (4000 paths)
    //   2. The gradient descent optimization converged to find the reserve price
    //   3. The option positions (vega, theta, delta) are within acceptable bounds
    let (simulate_price_verify_position_receipt, _simulate_price_verify_position_res) =
        simulate_price_verify_position_receipt(SimulatePriceVerifyPositionInput {
            start_timestamp,                                // Start of 90-day period
            end_timestamp,                                  // End of 90-day period
            data_length: data.len(),                        // Number of hourly data points (2160)
            positions: res.positions.clone(), // Optimized option positions (vega, theta, delta)
            pt: convert_array1_to_dvec(res.pt.clone()), // Markov transition matrix t
            pt_1: convert_array1_to_dvec(res.pt_1.clone()), // Markov transition matrix t-1
            gradient_tolerance,               // 5% tolerance for convergence
            // Residuals for price simulation
            de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
                res.de_seasonalised_detrended_log_base_fee.clone(),
            ),
            n_periods, // 720 three-hour periods
            num_paths, // 4000 Monte Carlo paths
            season_param: convert_array1_to_dvec(res.season_param.clone()), // Hourly seasonality
            twap_7d: res.twap_7d.clone(), // 7-day TWAP (mean reversion level)
            slope: res.slope, // Linear trend slope
            intercept: res.intercept, // Linear trend intercept
            reserve_price: res.reserve_price, // Calculated reserve price
            tolerance: reserve_price_tolerance, // 5% tolerance for reserve price
        });

    // ========== STEP 10: COMPOSE ALL INPUTS FOR FINAL ZK PROOF ==========
    // This aggregates all the previous computations into a single input structure
    // that will be used to generate the final composed ZK proof
    let input = ProofCompositionInput {
        // Hash of the 8-month data for on-chain verification
        data_8_months_hash: hashing_res.hash,
        // Full 8 months of hourly gas fee data
        data_8_months,
        // Timestamp range for the full 8-month dataset
        // Starts 5 months before the 3-month reserve price calculation period
        data_8_months_start_timestamp: start_timestamp - (3600 * 24 * 30 * 5),
        data_8_months_end_timestamp: end_timestamp,
        // Timestamp range for the 3-month reserve price calculation
        start_timestamp,
        end_timestamp,
        // Specific timestamp ranges for each calculation type
        twap_start_timestamp: start_timestamp,
        twap_end_timestamp: end_timestamp,
        reserve_price_start_timestamp: start_timestamp,
        reserve_price_end_timestamp: end_timestamp,
        max_return_start_timestamp: start_timestamp - (3600 * 24 * 30 * 5),
        max_return_end_timestamp: end_timestamp,
        // Results from reserve price calculation
        positions: res.positions,               // Optimized option positions
        pt: convert_array1_to_dvec(res.pt),     // Markov transition matrix t
        pt_1: convert_array1_to_dvec(res.pt_1), // Markov transition matrix t-1
        gradient_tolerance,                     // Convergence tolerance (5%)
        // Time series decomposition results
        de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
            res.de_seasonalised_detrended_log_base_fee,
        ),
        // Simulation parameters
        n_periods, // 720 three-hour periods
        num_paths, // 4000 Monte Carlo paths
        // Seasonal and trend parameters
        season_param: convert_array1_to_dvec(res.season_param), // 24 hourly values
        twap_7d: res.twap_7d,                                   // 7-day rolling TWAP
        slope: res.slope,                                       // Linear trend slope
        intercept: res.intercept,                               // Linear trend intercept
        reserve_price: res.reserve_price,                       // Final reserve price
        // Tolerances for verification
        floating_point_tolerance, // 0.00001% for intermediate calculations
        reserve_price_tolerance,  // 5% for reserve price
        // TWAP verification parameters
        twap_result: twap_original, // Expected TWAP value
        twap_tolerance: 1.0,        // 1% tolerance for TWAP
        // Maximum return (volatility measure)
        max_return: max_return_res.1, // Maximum return from historical data
    };

    // ========== STEP 11: BUILD EXECUTION ENVIRONMENT WITH PROOF ASSUMPTIONS ==========
    // The ExecutorEnv bundles all the individual proof receipts as "assumptions"
    // This enables proof composition: the final proof assumes the correctness of sub-proofs
    // without re-executing them, making the final proof smaller and faster to verify
    let env = ExecutorEnv::builder()
        // Assumption 1: Data hashing was performed correctly
        .add_assumption(hashing_receipt)
        // Assumption 2: TWAP calculation is within tolerance
        .add_assumption(calculate_twap_receipt)
        // Assumption 3: Maximum return was calculated correctly
        .add_assumption(max_return_receipt)
        // Assumption 4: Seasonality removal (time series decomposition) is correct
        .add_assumption(remove_seasonality_error_bound_receipt)
        // Assumption 5: 7-day TWAP calculation is within tolerance
        .add_assumption(add_twap_7d_error_bound_receipt)
        // Assumption 6: Markov transition probabilities (pt, pt_1) are correct
        .add_assumption(calculate_pt_pt1_error_bound_receipt)
        // Assumption 7: Price simulation and position verification are correct
        .add_assumption(simulate_price_verify_position_receipt)
        // Write the composed input data to the ZK-VM environment
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // ========== STEP 12: GENERATE THE COMPOSED ZK PROOF ==========
    // Execute the guest program (RISC Zero ZK-VM) to generate the proof
    // The guest program will verify all assumptions and produce a cryptographic receipt

    const MAX_RETRIES: u32 = 10;
    const INITIAL_DELAY_MS: u64 = 5000;

    let prover = default_prover();
    let mut last_error = None;
    let mut prove_info = None;

    for attempt in 1..=MAX_RETRIES {
        eprintln!(
            "proof_composition: Proof generation attempt {}/{}",
            attempt, MAX_RETRIES
        );

        match prover.prove(
            env.clone(), // Execution environment with all assumptions and inputs
            // The compiled guest ELF binary that runs in the ZK-VM
            PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_HASHING_GUEST_ELF,
        ) {
            Ok(info) => {
                eprintln!(
                    "proof_composition: Proof generation succeeded on attempt {}",
                    attempt
                );
                prove_info = Some(info);
                break;
            }
            Err(e) => {
                eprintln!(
                    "proof_composition: Attempt {}/{} failed: {}",
                    attempt, MAX_RETRIES, e
                );

                last_error = Some(e);

                if attempt == MAX_RETRIES {
                    // Final attempt - fail
                    break;
                }

                // Exponential backoff: 5s, 10s, 20s, 40s, etc.
                let delay = INITIAL_DELAY_MS * 2u64.pow(attempt - 1);
                eprintln!("proof_composition: Retrying in {}ms...", delay);
                thread::sleep(Duration::from_millis(delay));
            }
        }
    }

    let prove_info = prove_info.unwrap_or_else(|| {
        panic!(
            "proof_composition: Failed after {} attempts. Last error: {:?}",
            MAX_RETRIES,
            last_error.unwrap()
        )
    });

    // ========== STEP 13: VERIFY THE PROOF ==========
    // Extract the receipt (the ZK proof) from the proving result
    let receipt = prove_info.receipt;

    // Verify the proof using the guest program's unique identifier
    // This cryptographically verifies that:
    //   1. All 7 sub-proofs (assumptions) are valid
    //   2. The guest program executed correctly
    //   3. The reserve price calculation is correct within specified tolerances
    //   4. The data hash matches the input data
    // If verification succeeds, the proof can be submitted on-chain for trustless verification
    receipt
        .verify(PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_HASHING_GUEST_ID)
        .unwrap();
}
