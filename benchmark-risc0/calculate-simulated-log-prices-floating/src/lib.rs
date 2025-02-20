use calculate_simulated_log_prices_floating_core::CalculateSimulatedLogPricesInput;
use calculate_simulated_log_prices_floating_methods::CALCULATE_SIMULATED_LOG_PRICES_FLOATING_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub fn calculate_simulated_log_prices(
    input: CalculateSimulatedLogPricesInput,
) -> (Receipt, CalculateSimulatedLogPricesInput) {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    let prove_info = prover
        .prove(env, CALCULATE_SIMULATED_LOG_PRICES_FLOATING_GUEST_ELF)
        .unwrap();

    let receipt = prove_info.receipt;
    let res: CalculateSimulatedLogPricesInput = receipt.journal.decode().unwrap();

    (receipt, res)
}