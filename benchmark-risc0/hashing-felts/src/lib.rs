use hashing_felts_methods::HASHING_FELTS_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

use hashing_felts_core::{HashingFeltInput, HashingFeltOutput};

pub fn hash_felts(input: HashingFeltInput) -> (Receipt, HashingFeltOutput) {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let prove_info = prover.prove(env, HASHING_FELTS_GUEST_ELF).unwrap();

    let receipt = prove_info.receipt;
    let res: HashingFeltOutput = receipt.journal.decode().unwrap();

    (receipt, res)
}
