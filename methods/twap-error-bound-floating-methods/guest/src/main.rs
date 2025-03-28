use common::floating_point::{calculate_twap, error_bound_f64};
use core::TwapErrorBoundInput;
use risc0_zkvm::guest::env;

fn main() {
    let data: TwapErrorBoundInput = env::read();
    let twap_result = calculate_twap(&data.avg_hourly_gas_fee);

    let is_within_error_bound = error_bound_f64(twap_result, data.twap_result, data.twap_tolerance);
    assert!(is_within_error_bound);

    env::commit(&data);
}
