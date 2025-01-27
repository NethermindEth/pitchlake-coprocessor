use eyre::Result;
use risc0_zkvm::guest::env;

fn main() {
    let base_fee_per_gases: Vec<Option<f64>> = env::read();
    // Calculate log returns
    let mut returns: Vec<f64> = Vec::new();
    for i in 1..base_fee_per_gases.len() {
        if let (Some(ref basefee_current), Some(ref basefee_previous)) =
            (&base_fee_per_gases[i], &base_fee_per_gases[i - 1])
        {
            // Convert base fees from hex string to f64
            // let basefee_current = hex_string_to_f64(basefee_current).unwrap();
            // let basefee_previous = hex_string_to_f64(basefee_previous).unwrap();

            // If the previous base fee is zero, skip to the next iteration
            if *basefee_previous == 0.0 {
                continue;
            }

            // Calculate log return and add it to the returns vector
            returns.push((*basefee_current / *basefee_previous).ln());
        }
    }

    // If there are no returns the volatility is 0
    if returns.is_empty() {
        env::commit(&0f64);
        return;
    }

    // Calculate average returns
    let mean_return: f64 = returns.iter().sum::<f64>() / returns.len() as f64;

    // Calculate variance of average returns
    let variance: f64 = returns
        .iter()
        .map(|&r| (r - mean_return).powi(2))
        .sum::<f64>()
        / returns.len() as f64;

    env::commit(&(variance.sqrt() * 10_000.0).round());
}
