use core::VolatilityInputsFixedPointSimba;
use num_traits::Zero;
use risc0_zkvm::guest::env;
use simba::scalar::{ComplexField, FixedI48F16};

// TODO: to import from the `benchmark` crate
fn powi(x: FixedI48F16, y: usize) -> FixedI48F16 {
    let mut result = FixedI48F16::from_num(1);
    for _ in 0..y {
        result = result * x;
    }
    result
}

fn main() {
    let inputs: VolatilityInputsFixedPointSimba = env::read();
    let base_fee_per_gases: Vec<Option<FixedI48F16>> = inputs.base_fee_per_gases;
    let ln_results: Vec<FixedI48F16> = inputs.ln_results;

    let mut ln_result_counter = 0;
    for i in 1..base_fee_per_gases.len() {
        if let (Some(ref basefee_current), Some(ref basefee_previous)) =
            (&base_fee_per_gases[i], &base_fee_per_gases[i - 1])
        {
            if basefee_previous.is_zero() {
                continue;
            }

            let a = ln_results[ln_result_counter].exp() * *basefee_previous;
            let diff = a - *basefee_current;
            assert!(diff < FixedI48F16::from_num(0.00001));

            ln_result_counter += 1;
        }
    }

    // If there are no returns the volatility is 0
    if ln_results.is_empty() {
        env::commit(&FixedI48F16::zero());
        return;
    }

    let mean_return: FixedI48F16 = ln_results
        .iter()
        .fold(FixedI48F16::from_num(0), |acc, &x| acc + x)
        / FixedI48F16::from_num(ln_results.len());

    // Calculate variance of average returns
    let variance: FixedI48F16 = ln_results
        .iter()
        .map(|&r| powi(r - mean_return, 2))
        .fold(FixedI48F16::from_num(0), |acc, x| acc + x)
        / FixedI48F16::from_num(ln_results.len());

    let volatility: FixedI48F16 = variance.sqrt() * FixedI48F16::from_num(10000).round();
    env::commit(&volatility);
}
