use common::{convert_felt_to_f64, generate_batched_hash_for_all_avg_base_fees};
use core::{HashingFeltInput, HashingFeltOutput};
use risc0_zkvm::guest::env;

fn main() {
    let input: HashingFeltInput = env::read();

    // DEVELOPER NOTE: Fee Data Length Configuration
    // ============================================
    // Production configuration: 5760 hours (8 months of data)
    //   assert_eq!(input.inputs.len(), 5760);
    //
    // Current configuration: 1440 hours (2 months of data)
    // This reduced data requirement aligns with the POC configuration in:
    // - Cairo contract: starknet-contracts/fossil-hash-store/src/lib.cairo (num_in_a_batch = 8)
    // - Message handler: proving-service/crates/message-handler/src/proof_composition/mod.rs (REQUIRED_HOURS = 1440)
    //
    // IMPORTANT: When scaling to production (8 months), update this assertion to:
    //   assert_eq!(input.inputs.len(), 5760);
    // and ensure all related components are updated accordingly.
    assert_eq!(input.inputs.len(), 1440,
        "Expected 1440 hourly fee values for POC (2 months). Production requires 5760 (8 months).");

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
