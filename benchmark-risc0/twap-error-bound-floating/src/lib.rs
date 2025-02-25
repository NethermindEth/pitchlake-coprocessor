use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use twap_error_bound_floating_core::TwapErrorBoundInput;
use twap_error_bound_floating_methods::TWAP_ERROR_BOUND_FLOATING_GUEST_ELF;

pub fn calculate_twap(input: TwapErrorBoundInput) -> (Receipt, TwapErrorBoundInput) {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let prove_info = prover
        .prove(env, TWAP_ERROR_BOUND_FLOATING_GUEST_ELF)
        .unwrap();

    let receipt = prove_info.receipt;
    let res: TwapErrorBoundInput = receipt.journal.decode().unwrap();

    (receipt, res)
}
