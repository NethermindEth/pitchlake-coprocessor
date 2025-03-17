// use nalgebra::{DMatrix, DVector};
use add_twap_7d_error_bound_floating_core::AddTwap7dErrorBoundFloatingInput;
use add_twap_7d_error_bound_floating_methods::ADD_TWAP_7D_ERROR_BOUND_FLOATING_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub fn add_twap_7d_error_bound(
    input: AddTwap7dErrorBoundFloatingInput,
) -> (Receipt, AddTwap7dErrorBoundFloatingInput) {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let prove_info = prover
        .prove(env, ADD_TWAP_7D_ERROR_BOUND_FLOATING_GUEST_ELF)
        .unwrap();

    let receipt = prove_info.receipt;
    let res: AddTwap7dErrorBoundFloatingInput = receipt.journal.decode().unwrap();

    (receipt, res)
}
