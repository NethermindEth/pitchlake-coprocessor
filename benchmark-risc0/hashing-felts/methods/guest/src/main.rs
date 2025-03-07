use benchmark::generate_batched_hash_for_all_avg_base_fees;
use hashing_felts_core::HashingFeltInput;
use risc0_zkvm::guest::env;

fn main() {
    let input: HashingFeltInput = env::read();

    assert_eq!(input.inputs.len(), 5760);
    let hash_res = generate_batched_hash_for_all_avg_base_fees(&input.inputs);

    // convert felts to f64s
    let f64_inputs = input
        .inputs
        .iter()
        .map(|x| convert_felt_to_f64(*x))
        .collect::<Vec<_>>();

    env::commit(&HashingFeltOutput {
        hash: hash_res,
        f64_inputs,
    });
}
