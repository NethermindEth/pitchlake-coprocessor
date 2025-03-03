use add_twap_7d_error_bound_floating::add_twap_7d_error_bound;
use add_twap_7d_error_bound_floating_core::AddTwap7dErrorBoundFloatingInput;
use benchmark::{
    common::dataframe::{
        convert_to_timestamp_base_fee_int_tuple, filter_dataframe_by_date_range,
        read_data_from_file, replace_timestamp_with_date, split_dataframe_into_periods,
    },
    original::{self, convert_array1_to_dvec},
    tests::mock::{get_first_period_data, get_max_return_input_data},
};
use max_return_floating::max_return;
use max_return_floating_core::MaxReturnInput;
use proof_composition_twap_maxreturn_reserveprice_floating_core::ProofCompositionInput;
use proof_composition_twap_maxreturn_reserveprice_floating_methods::{
    PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_GUEST_ELF,
    PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_GUEST_ID,
};
use remove_seasonality_error_bound_floating::remove_seasonality_error_bound;
use remove_seasonality_error_bound_floating_core::RemoveSeasonalityErrorBoundFloatingInput;
use risc0_zkvm::{default_prover, ExecutorEnv};
use simulate_price_verify_position_floating::simulate_price_verify_position as simulate_price_verify_position_receipt;
use simulate_price_verify_position_floating_core::SimulatePriceVerifyPositionInput;

use twap_error_bound_floating::calculate_twap;
use twap_error_bound_floating_core::TwapErrorBoundInput;

use calculate_pt_pt1_error_bound_floating::calculate_pt_pt1_error_bound_floating;
use calculate_pt_pt1_error_bound_floating_core::CalculatePtPt1ErrorBoundFloatingInput;

fn main() {
    // max return
    let data_8_months = get_max_return_input_data();
    let input = MaxReturnInput {
        data: data_8_months.iter().map(|x| x.1).collect::<Vec<f64>>(),
    };
    let (max_return_receipt, max_return_res) = max_return(input);

    // twap
    // comparing twap using actual data from each block (host)
    // vs twap from using average hourly gas fee (zkvm)
    let df = read_data_from_file("data.csv");
    let df = replace_timestamp_with_date(df).unwrap();

    let data = data_8_months[data_8_months.len().saturating_sub(2160)..].to_vec();

    // calculate using original algorithm using base_fee for each block
    let raw_data_period = filter_dataframe_by_date_range(df, data[0].0, data[data.len() - 1].0);
    let raw_data: Vec<(i64, i64)> = convert_to_timestamp_base_fee_int_tuple(raw_data_period);
    let twap_original = original::calculate_twap::calculate_twap(&raw_data);
    println!("twap_original: {:?}", twap_original);
    let input = TwapErrorBoundInput {
        avg_hourly_gas_fee: data.iter().map(|x| x.1).collect::<Vec<f64>>(),
        twap_tolerance: 1.0,
        twap_result: twap_original,
    };

    let (calculate_twap_receipt, _calculate_twap_res) = calculate_twap(input);

    // reserve price
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
            data: data.clone(),
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
            data: data.clone(),
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

    let (simulate_price_verify_position_receipt, _simulate_price_verify_position_res) =
        simulate_price_verify_position_receipt(SimulatePriceVerifyPositionInput {
            data: data.clone(),
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
        });

    let input = ProofCompositionInput {
        data_8_months: data_8_months,
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
        twap_result: twap_original,
        twap_tolerance: 1.0,
        max_return: max_return_res.1,
    };

    let env = ExecutorEnv::builder()
        .add_assumption(calculate_twap_receipt)
        .add_assumption(max_return_receipt)
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
            PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_GUEST_ELF,
        )
        .unwrap();

    let receipt = prove_info.receipt;
    receipt
        .verify(PROOF_COMPOSITION_TWAP_MAXRETURN_RESERVEPRICE_FLOATING_GUEST_ID)
        .unwrap();
}
