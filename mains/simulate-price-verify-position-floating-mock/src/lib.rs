use core::SimulatePriceVerifyPositionInput;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use simulate_price_verify_position_floating_mock_methods::SIMULATE_PRICE_VERIFY_POSITION_FLOATING_MOCK_GUEST_ELF;

pub fn simulate_price_verify_position_mock(
    input: SimulatePriceVerifyPositionInput,
) -> (Receipt, SimulatePriceVerifyPositionInput) {
    eprintln!("simulate_price_verify_position_mock: Generating mock proof (no actual computation)");

    let prover = default_prover();

    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // Run the mock guest which just commits the input without computation
    let prove_info = prover
        .prove(env, SIMULATE_PRICE_VERIFY_POSITION_FLOATING_MOCK_GUEST_ELF)
        .expect("Failed to generate mock proof");

    let receipt = prove_info.receipt;
    let res: SimulatePriceVerifyPositionInput = receipt.journal.decode().unwrap();

    eprintln!("simulate_price_verify_position_mock: Mock proof generated successfully");

    (receipt, res)
}
