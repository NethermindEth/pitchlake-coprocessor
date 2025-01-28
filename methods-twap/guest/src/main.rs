// use core::{VolatilityInputsFixedPoint, VolatilityInputsFixedPointSimba};
// use fixed::types::I48F16;
// use num_traits::Zero;
// use risc0_zkvm::guest::env;
// use simba::scalar::{ComplexField, FixedI48F16, RealField};

// fn powi(x: FixedI48F16, y: usize) -> FixedI48F16 {
//     let mut result = FixedI48F16::from_num(1);
//     for _ in 0..y {
//         result = result * x;
//     }
//     result
// }

// fn main() {
//     let inputs: VolatilityInputsFixedPointSimba = env::read();
//     let base_fee_per_gases: Vec<Option<FixedI48F16>> = inputs.base_fee_per_gases;
//     let ln_results: Vec<FixedI48F16> = inputs.ln_results;

//     let mut ln_result_counter = 0;
//     for i in 1..base_fee_per_gases.len() {
//         if let (Some(ref basefee_current), Some(ref basefee_previous)) =
//             (&base_fee_per_gases[i], &base_fee_per_gases[i - 1])
//         {
//             if basefee_previous.is_zero() {
//                 continue;
//             }

//             let a = ln_results[ln_result_counter].exp() * *basefee_previous;
//             let diff = a - *basefee_current;
//             assert!(diff < FixedI48F16::from_num(0.00001));

//             ln_result_counter += 1;
//         }
//     }

//     // If there are no returns the volatility is 0
//     if ln_results.is_empty() {
//         env::commit(&FixedI48F16::zero());
//         return;
//     }

//     let mean_return: FixedI48F16 = ln_results
//         .iter()
//         .fold(FixedI48F16::from_num(0), |acc, &x| acc + x)
//         / FixedI48F16::from_num(ln_results.len());

//     // Calculate variance of average returns
//     let variance: FixedI48F16 = ln_results
//         .iter()
//         .map(|&r| powi(r - mean_return, 2))
//         .fold(FixedI48F16::from_num(0), |acc, x| acc + x)
//         / FixedI48F16::from_num(ln_results.len());

//     let volatility: FixedI48F16 = variance.sqrt() * FixedI48F16::from_num(10000).round();
//     env::commit(&volatility);
// }


use eth_rlp_types::BlockHeader;
use eyre::{anyhow, Result};
use risc0_zkvm::guest::env;

fn hex_string_to_f64(hex_str: &String) -> Result<f64> {
    let stripped = hex_str.trim_start_matches("0x");
    u128::from_str_radix(stripped, 16)
        .map(|value| value as f64)
        .map_err(|e| eyre::eyre!("Error converting hex string '{}' to f64: {}", hex_str, e))
}

fn main() {
    let headers: Vec<BlockHeader> = env::read();

    if headers.is_empty() {
        env::commit(&0.0);
        return;
    }

    let total_base_fee = headers.iter().try_fold(0.0, |acc, header| -> Result<f64> {
        let base_fee = header
            .base_fee_per_gas
            .clone()
            .unwrap_or_else(|| "0x0".to_string());
        let fee = hex_string_to_f64(&base_fee).unwrap();
        Ok(acc + fee)
    }).unwrap();

    let twap_result = total_base_fee / headers.len() as f64;
    env::commit(&twap_result);
}
