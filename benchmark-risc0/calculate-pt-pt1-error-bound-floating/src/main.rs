use benchmark::{
    original::{self, convert_array1_to_dvec},
    tests::mock::get_first_period_data,
};
use calculate_pt_pt1_error_bound_floating::calculate_pt_pt1_error_bound_floating;
use calculate_pt_pt1_error_bound_floating_core::CalculatePtPt1ErrorBoundFloatingInput;
use calculate_pt_pt1_error_bound_floating_methods::CALCULATE_PT_PT1_ERROR_BOUND_FLOATING_GUEST_ID;

fn main() {
    // get only first period of (timestamp avg_gas_fee)
    let data = get_first_period_data();
    // run rust code in host
    // ensure convergence in host
    let res = original::calculate_reserve_price(&data, 15000, 720);

    let input = CalculatePtPt1ErrorBoundFloatingInput {
        de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
            res.de_seasonalised_detrended_log_base_fee,
        ),
        pt: convert_array1_to_dvec(res.pt),
        pt_1: convert_array1_to_dvec(res.pt_1),
        tolerance: 0.00001, // 0.00001%
    };

    let (receipt, res) = calculate_pt_pt1_error_bound_floating(input);

    receipt
        .verify(CALCULATE_PT_PT1_ERROR_BOUND_FLOATING_GUEST_ID)
        .unwrap();
}
