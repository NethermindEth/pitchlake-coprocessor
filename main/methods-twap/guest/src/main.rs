use num_traits::Zero;
use risc0_zkvm::guest::env;
use simba::scalar::FixedI48F16;

fn main() {
    let base_fee_per_gases: Vec<FixedI48F16> = env::read();

    if base_fee_per_gases.is_empty() {
        env::commit(&FixedI48F16::zero());
        return;
    }

    let total_base_fee = base_fee_per_gases
        .iter()
        .fold(FixedI48F16::zero(), |acc, base_fee_per_gas| {
            acc + *base_fee_per_gas
        });

    let twap_result = total_base_fee / FixedI48F16::from_num(base_fee_per_gases.len());
    env::commit(&twap_result);
}
