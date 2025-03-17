// use nalgebra::{DMatrix, DVector};
use max_return_floating_core::MaxReturnInput;
use max_return_floating_methods::MAX_RETURN_FLOATING_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub fn max_return(input: MaxReturnInput) -> (Receipt, (MaxReturnInput, f64)) {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let prove_info = prover.prove(env, MAX_RETURN_FLOATING_GUEST_ELF).unwrap();

    let receipt = prove_info.receipt;
    let (input, max_return): (MaxReturnInput, f64) = receipt.journal.decode().unwrap();

    (receipt, (input, max_return))
}
