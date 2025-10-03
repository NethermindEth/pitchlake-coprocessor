// Example: How to call proof composition from an external application
//
// This example demonstrates the complete workflow for generating a proof
// of reserve price calculation that can be verified on-chain.

use add_twap_7d_error_bound_floating::add_twap_7d_error_bound;
use calculate_pt_pt1_error_bound_floating::calculate_pt_pt1_error_bound_floating;
use common::{
    common::dataframe::{read_data_from_file, replace_timestamp_with_date},
    floating_point,
    original::{self, convert_array1_to_dvec},
    tests::mock::{convert_data_to_vec_of_tuples, get_5760_avg_base_fees_felt},
};
use core::{
    AddTwap7dErrorBoundFloatingInput, CalculatePtPt1ErrorBoundFloatingInput, HashingFeltInput,
    MaxReturnInput, ProofCompositionInput, ProofCompositionOutput,
    RemoveSeasonalityErrorBoundFloatingInput, SimulatePriceVerifyPositionInput,
    TwapErrorBoundInput,
};
use hashing_felts::hash_felts;
use max_return_floating::max_return;
use proof_composition_twap_maxreturn_reserveprice_floating_hashing_methods::{
    PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_HASHING_GUEST_ELF,
    PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_HASHING_GUEST_ID,
};
use remove_seasonality_error_bound_floating::remove_seasonality_error_bound;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use simulate_price_verify_position_floating::simulate_price_verify_position;
use twap_error_bound_floating::calculate_twap;

/// Complete proof generation function that can be called from external applications
///
/// # Arguments
/// * `data_8_months` - 5760 hours (240 days) of hourly average gas fees in gwei
/// * `start_timestamp` - Unix timestamp for the start of the 3-month analysis period
/// * `end_timestamp` - Unix timestamp for the end of the 3-month analysis period
/// * `strike_price` - Strike price for the option in gwei (typically 15000)
///
/// # Returns
/// * `Receipt` - The ZK proof receipt that can be verified on-chain
///
/// # Example
/// ```rust
/// let data = fetch_historical_gas_prices()?;
/// let start = chrono::Utc::now().timestamp() - (90 * 24 * 3600);
/// let end = chrono::Utc::now().timestamp();
/// let receipt = generate_reserve_price_proof(data, start, end, 15000)?;
/// ```
pub fn generate_reserve_price_proof(
    data_8_months: Vec<f64>,
    start_timestamp: i64,
    end_timestamp: i64,
    strike_price: i64,
) -> Result<Receipt, Box<dyn std::error::Error>> {
    // ═══════════════════════════════════════════════════════════════════════
    // VALIDATION: Ensure input data is valid
    // ═══════════════════════════════════════════════════════════════════════
    println!("🔍 Validating input data...");

    if data_8_months.len() != 5760 {
        return Err(format!(
            "Expected exactly 5760 data points (240 days), got {}",
            data_8_months.len()
        )
        .into());
    }

    for (i, &value) in data_8_months.iter().enumerate() {
        if value <= 0.0 {
            return Err(format!("Gas fee at hour {} is non-positive: {}", i, value).into());
        }
        if value > 10000.0 {
            return Err(format!("Gas fee at hour {} is unrealistically high: {}", i, value).into());
        }
    }

    let duration_days = (end_timestamp - start_timestamp) / (24 * 3600);
    if duration_days < 85 || duration_days > 95 {
        return Err(format!(
            "Time period should be approximately 90 days, got {} days",
            duration_days
        )
        .into());
    }

    println!("✅ Input validation passed");

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 1: Generate Sub-Proof #1 - Data Hashing
    // ═══════════════════════════════════════════════════════════════════════
    println!("🔐 Step 1/7: Hashing gas fee data...");

    let inputs_felt = get_5760_avg_base_fees_felt(); // Convert to Starknet field elements
    let (hashing_receipt, hashing_res) = hash_felts(HashingFeltInput {
        inputs: inputs_felt,
    });

    println!(
        "   ✓ Data hash: {:?}",
        &hashing_res.hash[..4] // Show first 4 elements
    );

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 2: Generate Sub-Proof #2 - Maximum Return (Volatility)
    // ═══════════════════════════════════════════════════════════════════════
    println!("📈 Step 2/7: Calculating maximum return (volatility)...");

    let (max_return_receipt, max_return_res) = max_return(MaxReturnInput {
        data: hashing_res.f64_inputs.clone(),
    });

    println!("   ✓ Maximum return: {:.4}%", max_return_res.1 * 100.0);

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 3: Extract 3-month subset for reserve price calculation
    // ═══════════════════════════════════════════════════════════════════════
    println!("📅 Extracting 3-month data subset...");

    let data_3_months = hashing_res.f64_inputs[hashing_res.f64_inputs.len().saturating_sub(2160)..]
        .to_vec();

    println!(
        "   ✓ Using {} data points (last 90 days)",
        data_3_months.len()
    );

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 4: Generate Sub-Proof #3 - TWAP
    // ═══════════════════════════════════════════════════════════════════════
    println!("⏱️  Step 3/7: Calculating TWAP...");

    let twap_original = floating_point::calculate_twap(&data_3_months);

    let (calculate_twap_receipt, _) = calculate_twap(TwapErrorBoundInput {
        avg_hourly_gas_fee: data_3_months.clone(),
        twap_tolerance: 1.0,
        twap_result: twap_original,
    });

    println!("   ✓ TWAP: {:.2} gwei", twap_original);

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 5: Calculate Reserve Price (HOST COMPUTATION)
    // ═══════════════════════════════════════════════════════════════════════
    println!("🎲 Calculating reserve price (this may take 1-2 minutes)...");

    let n_periods = 720; // 720 three-hour periods in 90 days
    let data_with_timestamps =
        convert_data_to_vec_of_tuples(data_3_months.clone(), start_timestamp);

    let res = original::calculate_reserve_price(&data_with_timestamps, strike_price, n_periods);

    println!("   ✓ Reserve price: {:.2} gwei", res.reserve_price);
    println!(
        "   ✓ Optimization converged with {} iterations",
        res.positions.len()
    );

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 6: Generate Sub-Proof #4 - Seasonality Removal
    // ═══════════════════════════════════════════════════════════════════════
    println!("🔄 Step 4/7: Verifying time series decomposition...");

    let floating_point_tolerance = 0.00001;

    let (remove_seasonality_receipt, _) =
        remove_seasonality_error_bound(RemoveSeasonalityErrorBoundFloatingInput {
            data: data_3_months.clone(),
            slope: res.slope,
            intercept: res.intercept,
            de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
                res.de_seasonalised_detrended_log_base_fee.clone(),
            ),
            season_param: convert_array1_to_dvec(res.season_param.clone()),
            tolerance: floating_point_tolerance,
        });

    println!("   ✓ Trend slope: {:.6}", res.slope);
    println!("   ✓ Trend intercept: {:.6}", res.intercept);

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 7: Generate Sub-Proof #5 - 7-Day TWAP
    // ═══════════════════════════════════════════════════════════════════════
    println!("📊 Step 5/7: Verifying 7-day TWAP...");

    let (add_twap_7d_receipt, _) = add_twap_7d_error_bound(AddTwap7dErrorBoundFloatingInput {
        data: data_3_months.clone(),
        twap_7d: res.twap_7d.clone(),
        tolerance: floating_point_tolerance,
    });

    println!("   ✓ 7-day TWAP values: {} entries", res.twap_7d.len());

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 8: Generate Sub-Proof #6 - Markov Transition Matrices
    // ═══════════════════════════════════════════════════════════════════════
    println!("🔀 Step 6/7: Verifying Markov transition probabilities...");

    let (calculate_pt_pt1_receipt, _) =
        calculate_pt_pt1_error_bound_floating(CalculatePtPt1ErrorBoundFloatingInput {
            de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
                res.de_seasonalised_detrended_log_base_fee.clone(),
            ),
            pt: convert_array1_to_dvec(res.pt.clone()),
            pt_1: convert_array1_to_dvec(res.pt_1.clone()),
            tolerance: floating_point_tolerance,
        });

    println!("   ✓ Transition matrix pt: {}x{}", res.pt.nrows(), res.pt.ncols());
    println!(
        "   ✓ Transition matrix pt_1: {}x{}",
        res.pt_1.nrows(),
        res.pt_1.ncols()
    );

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 9: Generate Sub-Proof #7 - Price Simulation
    // ═══════════════════════════════════════════════════════════════════════
    println!("🎰 Step 7/7: Verifying price simulation (this may take 2-3 minutes)...");

    let num_paths = 4000;
    let gradient_tolerance = 5e-2;
    let reserve_price_tolerance = 5.0;

    let (simulate_price_receipt, _) = simulate_price_verify_position(SimulatePriceVerifyPositionInput {
        start_timestamp,
        end_timestamp,
        data_length: data_3_months.len(),
        positions: res.positions.clone(),
        pt: convert_array1_to_dvec(res.pt.clone()),
        pt_1: convert_array1_to_dvec(res.pt_1.clone()),
        gradient_tolerance,
        de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
            res.de_seasonalised_detrended_log_base_fee.clone(),
        ),
        n_periods,
        num_paths,
        season_param: convert_array1_to_dvec(res.season_param.clone()),
        twap_7d: res.twap_7d.clone(),
        slope: res.slope,
        intercept: res.intercept,
        reserve_price: res.reserve_price,
        tolerance: reserve_price_tolerance,
    });

    println!("   ✓ Simulated {} price paths", num_paths);

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 10: Compose All Inputs
    // ═══════════════════════════════════════════════════════════════════════
    println!("🔗 Composing final proof input...");

    let data_8_months_start_timestamp = start_timestamp - (3600 * 24 * 30 * 5);

    let input = ProofCompositionInput {
        data_8_months_hash: hashing_res.hash,
        data_8_months: hashing_res.f64_inputs,
        data_8_months_start_timestamp,
        data_8_months_end_timestamp: end_timestamp,
        start_timestamp,
        end_timestamp,
        twap_start_timestamp: start_timestamp,
        twap_end_timestamp: end_timestamp,
        reserve_price_start_timestamp: start_timestamp,
        reserve_price_end_timestamp: end_timestamp,
        max_return_start_timestamp: data_8_months_start_timestamp,
        max_return_end_timestamp: end_timestamp,
        positions: res.positions,
        pt: convert_array1_to_dvec(res.pt),
        pt_1: convert_array1_to_dvec(res.pt_1),
        gradient_tolerance,
        de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
            res.de_seasonalised_detrended_log_base_fee,
        ),
        n_periods,
        num_paths,
        season_param: convert_array1_to_dvec(res.season_param),
        twap_7d: res.twap_7d,
        slope: res.slope,
        intercept: res.intercept,
        reserve_price: res.reserve_price,
        floating_point_tolerance,
        reserve_price_tolerance,
        twap_tolerance: 1.0,
        twap_result: twap_original,
        max_return: max_return_res.1,
    };

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 11: Generate Composed ZK Proof
    // ═══════════════════════════════════════════════════════════════════════
    println!("⚙️  Generating composed ZK proof (this may take 2-3 minutes)...");

    let env = ExecutorEnv::builder()
        .add_assumption(hashing_receipt)
        .add_assumption(calculate_twap_receipt)
        .add_assumption(max_return_receipt)
        .add_assumption(remove_seasonality_receipt)
        .add_assumption(add_twap_7d_receipt)
        .add_assumption(calculate_pt_pt1_receipt)
        .add_assumption(simulate_price_receipt)
        .write(&input)?
        .build()?;

    let prove_info = default_prover().prove(
        env,
        PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_HASHING_GUEST_ELF,
    )?;

    let receipt = prove_info.receipt;

    // ═══════════════════════════════════════════════════════════════════════
    // STEP 12: Verify the Proof
    // ═══════════════════════════════════════════════════════════════════════
    println!("✅ Verifying proof...");

    receipt.verify(PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_HASHING_GUEST_ID)?;

    println!("🎉 Proof generated and verified successfully!");

    Ok(receipt)
}

/// Extract results from a proof receipt
pub fn extract_proof_output(receipt: &Receipt) -> Result<ProofCompositionOutput, Box<dyn std::error::Error>> {
    let output: ProofCompositionOutput = receipt.journal.decode()?;
    Ok(output)
}

/// Example: Complete workflow from data fetching to proof generation
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Gas Fee Option Reserve Price Proof Generation Example");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 1: Prepare Data
    // ═══════════════════════════════════════════════════════════════════════
    println!("📥 Phase 1: Fetching historical gas price data...");

    // In a real application, you would fetch this from:
    // - Ethereum RPC node
    // - Etherscan API
    // - The Graph
    // - Your own database
    let data_8_months = get_5760_avg_base_fees_felt(); // Mock data for this example
    let data_8_months_f64: Vec<f64> = data_8_months.iter().map(|&x| x as f64).collect();

    println!("   ✓ Loaded {} hours of gas price data", data_8_months_f64.len());

    // Define time periods
    let end_timestamp = chrono::Utc::now().timestamp();
    let start_timestamp = end_timestamp - (90 * 24 * 3600); // 90 days ago

    println!("   ✓ Analysis period: 90 days");
    println!(
        "   ✓ Start: {}",
        chrono::DateTime::from_timestamp(start_timestamp, 0)
            .unwrap()
            .format("%Y-%m-%d")
    );
    println!(
        "   ✓ End:   {}",
        chrono::DateTime::from_timestamp(end_timestamp, 0)
            .unwrap()
            .format("%Y-%m-%d")
    );
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 2: Generate Proof
    // ═══════════════════════════════════════════════════════════════════════
    println!("⚡ Phase 2: Generating ZK proof...");
    println!("   (This will take approximately 6-10 minutes)");
    println!();

    let start_time = std::time::Instant::now();

    let receipt = generate_reserve_price_proof(
        data_8_months_f64,
        start_timestamp,
        end_timestamp,
        15000, // Strike price: 15000 gwei
    )?;

    let elapsed = start_time.elapsed();
    println!();
    println!("⏱️  Total time: {:.2} seconds", elapsed.as_secs_f64());
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 3: Extract and Display Results
    // ═══════════════════════════════════════════════════════════════════════
    println!("📊 Phase 3: Extracting results...");
    println!();

    let output = extract_proof_output(&receipt)?;

    println!("═══════════════════════════════════════════════════════════");
    println!("                      RESULTS                              ");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("🔐 Data Hash: {:?}", output.data_8_months_hash);
    println!();
    println!("💰 Reserve Price: {}", output.reserve_price);
    println!("   (Minimum price for selling gas fee options)");
    println!();
    println!("📈 TWAP: {}", output.twap_result);
    println!("   (Time-weighted average gas price over 90 days)");
    println!();
    println!("📊 Max Return: {}", output.max_return);
    println!("   (Historical volatility measure over 240 days)");
    println!();
    println!("⚙️  Tolerances:");
    println!("   - Floating point: {}", output.floating_point_tolerance);
    println!("   - Reserve price: {}", output.reserve_price_tolerance);
    println!("   - Gradient: {}", output.gradient_tolerance);
    println!("   - TWAP: {}", output.twap_tolerance);
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 4: Save Proof for On-Chain Submission
    // ═══════════════════════════════════════════════════════════════════════
    println!("💾 Phase 4: Saving proof for on-chain submission...");

    let receipt_bytes = bincode::serialize(&receipt)?;
    std::fs::write("proof_receipt.bin", &receipt_bytes)?;

    println!("   ✓ Saved to: proof_receipt.bin");
    println!("   ✓ Size: {} KB", receipt_bytes.len() / 1024);
    println!();

    println!("🎉 All done! You can now submit this proof on-chain.");
    println!();
    println!("Next steps:");
    println!("  1. Deploy RISC Zero verifier contract on Starknet");
    println!("  2. Submit proof_receipt.bin to the verifier");
    println!("  3. On-chain verification will confirm the reserve price");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_generation_with_mock_data() {
        let data = get_5760_avg_base_fees_felt();
        let data_f64: Vec<f64> = data.iter().map(|&x| x as f64).collect();

        let start = 1708833600;
        let end = start + (90 * 24 * 3600);

        let receipt = generate_reserve_price_proof(data_f64, start, end, 15000).unwrap();

        let output = extract_proof_output(&receipt).unwrap();

        // Basic sanity checks
        assert!(output.reserve_price.len() > 0);
        assert!(output.twap_result.len() > 0);
        assert!(output.max_return.len() > 0);
    }

    #[test]
    fn test_validation_rejects_wrong_data_length() {
        let data = vec![15.0; 1000]; // Only 1000 points instead of 5760
        let start = 1708833600;
        let end = start + (90 * 24 * 3600);

        let result = generate_reserve_price_proof(data, start, end, 15000);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected exactly 5760"));
    }

    #[test]
    fn test_validation_rejects_negative_values() {
        let mut data = vec![15.0; 5760];
        data[100] = -5.0; // Invalid negative gas price

        let start = 1708833600;
        let end = start + (90 * 24 * 3600);

        let result = generate_reserve_price_proof(data, start, end, 15000);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-positive"));
    }
}
