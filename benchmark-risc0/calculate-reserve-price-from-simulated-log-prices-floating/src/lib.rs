use calculate_reserve_price_from_simulated_log_prices_floating_core::CalculateReservePriceFromSimulatedLogPricesInput;
use calculate_reserve_price_from_simulated_log_prices_floating_methods::CALCULATE_RESERVE_PRICE_FROM_SIMULATED_LOG_PRICES_FLOATING_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub fn calculate_reserve_price_from_simulated_log_prices(
    input: CalculateReservePriceFromSimulatedLogPricesInput,
) -> (
    Receipt,
    (CalculateReservePriceFromSimulatedLogPricesInput, f64),
) {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    let prove_info = prover
        .prove(
            env,
            CALCULATE_RESERVE_PRICE_FROM_SIMULATED_LOG_PRICES_FLOATING_GUEST_ELF,
        )
        .unwrap();

    let receipt = prove_info.receipt;
    let res: (CalculateReservePriceFromSimulatedLogPricesInput, f64) =
        receipt.journal.decode().unwrap();

    (receipt, res)
}
