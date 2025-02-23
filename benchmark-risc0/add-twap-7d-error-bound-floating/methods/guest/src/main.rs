use add_twap_7d_error_bound_floating_core::AddTwap7dErrorBoundFloatingInput;
use benchmark::floating_point::{add_twap_7d, error_bound_vec};
use risc0_zkvm::guest::env;

fn main() {
    let input: AddTwap7dErrorBoundFloatingInput = env::read();
    let res = add_twap_7d(&input.data).unwrap();

    let is_within_tolerance = error_bound_vec(&input.twap_7d, &res, input.tolerance);
    assert!(is_within_tolerance);

    env::commit(&input);
}
