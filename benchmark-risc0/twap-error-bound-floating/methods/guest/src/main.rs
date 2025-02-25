use risc0_zkvm::guest::env;

fn main() {
    // let base_fee_per_gases: Vec<FixedI48F16> = env::read();

    // if base_fee_per_gases.is_empty() {
    //     env::commit(&FixedI48F16::zero());
    //     return;
    // }

    // let total_base_fee = base_fee_per_gases
    //     .iter()
    //     .fold(FixedI48F16::zero(), |acc, base_fee_per_gas| {
    //         acc + *base_fee_per_gas
    //     });

    // let twap_result = total_base_fee / FixedI48F16::from_num(base_fee_per_gases.len());
    // env::commit(&twap_result);
}

// pub fn calculate_twap(headers: Vec<BlockHeader>) -> Result<f64> {
//     if headers.is_empty() {
//         return Err(anyhow!("The provided block headers are empty."));
//     }

//     let total_base_fee = headers.iter().try_fold(0.0, |acc, header| -> Result<f64> {
//         let base_fee = header
//             .base_fee_per_gas
//             .clone()
//             .unwrap_or_else(|| "0x0".to_string());
//         let fee = hex_string_to_f64(&base_fee)?;
//         Ok(acc + fee)
//     })?;

//     let twap_result = total_base_fee / headers.len() as f64;

//     Ok(twap_result)
// }
