use calculate_pt_pt1_error_bound_floating_methods::CALCULATE_PT_PT1_ERROR_BOUND_FLOATING_GUEST_ELF;
use core::CalculatePtPt1ErrorBoundFloatingInput;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub fn calculate_pt_pt1_error_bound_floating(
    input: CalculatePtPt1ErrorBoundFloatingInput,
) -> (Receipt, CalculatePtPt1ErrorBoundFloatingInput) {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let prove_info = prover
        .prove(env, CALCULATE_PT_PT1_ERROR_BOUND_FLOATING_GUEST_ELF)
        .unwrap();

    let receipt = prove_info.receipt;
    let res: CalculatePtPt1ErrorBoundFloatingInput = receipt.journal.decode().unwrap();

    (receipt, res)
}
