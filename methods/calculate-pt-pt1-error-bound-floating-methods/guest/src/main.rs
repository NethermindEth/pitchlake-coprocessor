use common::floating_point::{error_bound_dvec, pre_minimize};
use core::CalculatePtPt1ErrorBoundFloatingInput;
use risc0_zkvm::guest::env;

fn main() {
    let input: CalculatePtPt1ErrorBoundFloatingInput = env::read();
    let (pt, pt_1, _var_pt) = pre_minimize(&input.de_seasonalised_detrended_log_base_fee);

    let is_within_tolerance_pt = error_bound_dvec(&pt, &input.pt, input.tolerance);
    assert!(is_within_tolerance_pt);

    let is_within_tolerance_pt_1 = error_bound_dvec(&pt_1, &input.pt_1, input.tolerance);
    assert!(is_within_tolerance_pt_1);

    env::commit(&input);
}