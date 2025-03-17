use nalgebra::{DMatrix, DVector};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use simulate_price_verify_position_floating_core::SimulatePriceVerifyPositionInput;
use simulate_price_verify_position_floating_methods::SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ELF;

pub fn simulate_price_verify_position(
    input: SimulatePriceVerifyPositionInput,
) -> (Receipt, SimulatePriceVerifyPositionInput) {
    let env = ExecutorEnv::builder()
        // Send a & b to the guest
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let prove_info = prover
        .prove(env, SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ELF)
        .unwrap();

    let receipt = prove_info.receipt;
    let res: SimulatePriceVerifyPositionInput = receipt.journal.decode().unwrap();

    (receipt, res)
}
