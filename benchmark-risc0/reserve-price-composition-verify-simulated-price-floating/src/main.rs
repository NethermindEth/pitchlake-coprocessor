use benchmark::{
    original::{self, convert_array1_to_dvec, convert_array2_to_dmatrix},
    tests::mock::get_first_period_data,
};
use reserve_price_composition_verify_simulated_price_floating_core::ReservePriceCompositionInput;
use reserve_price_composition_verify_simulated_price_floating_methods::{
    RESERVE_PRICE_COMPOSITION_VERIFY_SIMULATED_PRICE_FLOATING_GUEST_ELF,
    RESERVE_PRICE_COMPOSITION_VERIFY_SIMULATED_PRICE_FLOATING_GUEST_ID,
};
use risc0_zkvm::{default_prover, ExecutorEnv};
use simulate_price_verify_position_floating::simulate_price_verify_position as simulate_price_verify_position_receipt;
use simulate_price_verify_position_floating_core::SimulatePriceVerifyPositionInput;

fn main() {
    let data = get_first_period_data();
    // run rust code in host
    // ensure convergence in host
    let num_paths = 4000;
    let n_periods = 720;
    let res = original::calculate_reserve_price(&data, num_paths, n_periods);

    let gradient_tolerance = 5e-3;

    let (simulate_price_verify_position_receipt, simulate_price_verify_position_res) =
        simulate_price_verify_position_receipt(SimulatePriceVerifyPositionInput {
            positions: res.positions.clone(),
            pt: convert_array1_to_dvec(res.pt.clone()),
            pt_1: convert_array1_to_dvec(res.pt_1.clone()),
            gradient_tolerance,
            de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
                res.de_seasonalised_detrended_log_base_fee.clone(),
            ),
            n_periods,
            num_paths,
        });

    let input = ReservePriceCompositionInput {
        positions: res.positions,
        pt: convert_array1_to_dvec(res.pt),
        pt_1: convert_array1_to_dvec(res.pt_1),
        gradient_tolerance,
        de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
            res.de_seasonalised_detrended_log_base_fee,
        ),
        n_periods,
        num_paths,
        // de_seasonalized_detrended_simulated_prices: convert_array2_to_dmatrix(
        //     res.de_seasonalized_detrended_simulated_prices,
        // ),
        de_seasonalized_detrended_simulated_prices: simulate_price_verify_position_res.1, // todo: change this so that we can pass this in from AllInput, do the error bound check
    };

    let env = ExecutorEnv::builder()
        .add_assumption(simulate_price_verify_position_receipt)
        // .add_assumption(simulate_price_receipt)
        // .add_assumption(add_twap_7d_receipt)
        // .add_assumption(calculate_reserve_price_receipt)
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
