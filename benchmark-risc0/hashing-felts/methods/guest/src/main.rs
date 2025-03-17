use common::{convert_felt_to_f64, generate_batched_hash_for_all_avg_base_fees};
use hashing_felts_core::{HashingFeltInput, HashingFeltOutput};
use risc0_zkvm::guest::env;

fn main() {
    let input: HashingFeltInput = env::read();

    assert_eq!(input.inputs.len(), 5760);
    let hash_res = generate_batched_hash_for_all_avg_base_fees(&input.inputs);

    let mut u32_result = [0u32; 8];
    for i in 0..8 {
        u32_result[i] = u32::from_be_bytes(hash_res[i * 4..(i + 1) * 4].try_into().unwrap());
    }

    // convert felts to f64s
    let f64_inputs = input
        .inputs
        .iter()
        .map(|x| convert_felt_to_f64(*x))
        .collect::<Vec<_>>();

    env::commit(&HashingFeltOutput {
        hash: u32_result,
        f64_inputs,
    });
}
