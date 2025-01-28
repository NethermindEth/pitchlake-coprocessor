use eyre::Result;
use risc0_zkvm::guest::env;

fn main() {
    let base_fee_per_gases: Vec<f64> = env::read();

    if base_fee_per_gases.is_empty() {
        env::commit(&0.0);
        return;
    }

    let total_base_fee = base_fee_per_gases
        .iter()
        .try_fold(0.0, |acc, base_fee_per_gas| -> Result<f64> {
            Ok(acc + base_fee_per_gas)
        })
        .unwrap();

    let twap_result = total_base_fee / base_fee_per_gases.len() as f64;
    env::commit(&twap_result);
}
