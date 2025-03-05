use add_twap_7d_error_bound_floating::add_twap_7d_error_bound;
use add_twap_7d_error_bound_floating_core::AddTwap7dErrorBoundFloatingInput;
use benchmark::{
    original::{self, convert_array1_to_dvec},
    tests::mock::get_first_period_data,
};
use remove_seasonality_error_bound_floating::remove_seasonality_error_bound;
use remove_seasonality_error_bound_floating_core::RemoveSeasonalityErrorBoundFloatingInput;
use reserve_price_composition_verify_simulated_price_floating_core::ReservePriceCompositionInput;
use reserve_price_composition_verify_simulated_price_floating_methods::{
    RESERVE_PRICE_COMPOSITION_VERIFY_SIMULATED_PRICE_FLOATING_GUEST_ELF,
    RESERVE_PRICE_COMPOSITION_VERIFY_SIMULATED_PRICE_FLOATING_GUEST_ID,
};
use risc0_zkvm::{default_prover, ExecutorEnv};
use simulate_price_verify_position_floating::simulate_price_verify_position as simulate_price_verify_position_receipt;
use simulate_price_verify_position_floating_core::SimulatePriceVerifyPositionInput;

use calculate_pt_pt1_error_bound_floating::calculate_pt_pt1_error_bound_floating;
use calculate_pt_pt1_error_bound_floating_core::CalculatePtPt1ErrorBoundFloatingInput;

fn main() {
    let data = get_first_period_data();
    // run rust code in host
    // ensure convergence in host
    let n_periods = 720;
    let res = original::calculate_reserve_price(&data, 15000, n_periods);

    let num_paths = 4000;
    let gradient_tolerance = 5e-3;
    let floating_point_tolerance = 0.00001; // 0.00001%
    let reserve_price_tolerance = 5.0; // 5%

    let (remove_seasonality_error_bound_receipt, _remove_seasonality_error_bound_res) =
        remove_seasonality_error_bound(RemoveSeasonalityErrorBoundFloatingInput {
            data: data.clone().iter().map(|x| x.1).collect(),
            slope: res.slope,
            intercept: res.intercept,
            de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
                res.de_seasonalised_detrended_log_base_fee.clone(),
            ),
            season_param: convert_array1_to_dvec(res.season_param.clone()),
            tolerance: floating_point_tolerance,
        });

    let (add_twap_7d_error_bound_receipt, _add_twap_7d_error_bound_res) =
        add_twap_7d_error_bound(AddTwap7dErrorBoundFloatingInput {
            data: data.clone().iter().map(|x| x.1).collect(),
            twap_7d: res.twap_7d.clone(),
            tolerance: floating_point_tolerance,
        });

    let (calculate_pt_pt1_error_bound_receipt, _calculate_pt_pt1_error_bound_res) =
        calculate_pt_pt1_error_bound_floating(CalculatePtPt1ErrorBoundFloatingInput {
            de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
                res.de_seasonalised_detrended_log_base_fee.clone(),
            ),
            pt: convert_array1_to_dvec(res.pt.clone()),
            pt_1: convert_array1_to_dvec(res.pt_1.clone()),
            tolerance: floating_point_tolerance,
        });

    let (simulate_price_verify_position_receipt, simulate_price_verify_position_res) =
        simulate_price_verify_position_receipt(SimulatePriceVerifyPositionInput {
            start_timestamp: data[0].0,
            end_timestamp: data[data.len() - 1].0,
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
            tolerance: reserve_price_tolerance, // 5%
            data_length: data.len(),
        });

    let input = ReservePriceCompositionInput {
        data: data.iter().map(|x| x.1).collect(),
        start_timestamp: data[0].0,
        end_timestamp: data[data.len() - 1].0,
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
    };

    let env = ExecutorEnv::builder()
        .add_assumption(remove_seasonality_error_bound_receipt)
        .add_assumption(add_twap_7d_error_bound_receipt)
        .add_assumption(calculate_pt_pt1_error_bound_receipt)
        .add_assumption(simulate_price_verify_position_receipt)
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    let prove_info = default_prover()
        .prove(
            env,
            RESERVE_PRICE_COMPOSITION_VERIFY_SIMULATED_PRICE_FLOATING_GUEST_ELF,
        )
        .unwrap();

    let receipt = prove_info.receipt;
    receipt
        .verify(RESERVE_PRICE_COMPOSITION_VERIFY_SIMULATED_PRICE_FLOATING_GUEST_ID)
        .unwrap();
}
