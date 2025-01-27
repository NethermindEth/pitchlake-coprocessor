use risc0_zkvm::guest::env;
use core::VolatilityInputs;

fn main() {
    let inputs: VolatilityInputs = env::read();
    let base_fee_per_gases: Vec<Option<f64>> = inputs.base_fee_per_gases;
    let ln_results: Vec<f64> = inputs.ln_results;

    // let base_fee_per_gases: Vec<Option<f64>> = env::read();

    // // Calculate log returns
    // let mut ln_results: Vec<f64> = Vec::new();
    // for i in 1..base_fee_per_gases.len() {
    //     if let (Some(ref basefee_current), Some(ref basefee_previous)) =
    //         (&base_fee_per_gases[i], &base_fee_per_gases[i - 1])
    //     {
    //         // If the previous base fee is zero, skip to the next iteration
    //         if *basefee_previous == 0.0 {
    //             continue;
    //         }

    //         // Calculate log return and add it to the returns vector
    //         ln_results.push((*basefee_current / *basefee_previous).ln());

    //         // // approach 2: 4528 cycles
    //         // let a = (&input_and_result.result.exp() * denominator);
    //         // let diff = a - numerator;
    //         // assert!(diff < 0.00001);
    //     }
    // }

    let mut ln_result_counter = 0;
    for i in 1..base_fee_per_gases.len() {
        if let (Some(ref basefee_current), Some(ref basefee_previous)) =
            (&base_fee_per_gases[i], &base_fee_per_gases[i - 1])
        {
            if *basefee_previous == 0.0 {
                continue;
            }

            let a = ln_results[ln_result_counter].exp() * *basefee_previous;
            let diff = a - *basefee_current;
            assert!(diff < 0.00001);

            ln_result_counter += 1;
        }
    }

    // If there are no returns the volatility is 0
    if ln_results.is_empty() {
        env::commit(&0f64);
        return;
    }

    // Calculate average returns
    let mean_return: f64 = ln_results.iter().sum::<f64>() / ln_results.len() as f64;

    // Calculate variance of average returns
    let variance: f64 = ln_results
        .iter()
        .map(|&r| (r - mean_return).powi(2))
        .sum::<f64>()
        / ln_results.len() as f64;

    env::commit(&(variance.sqrt() * 10_000.0).round());
}
