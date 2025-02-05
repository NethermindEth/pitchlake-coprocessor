use benchmark::fixed_point::FixedPoint;
use nalgebra::{DMatrix, DVector};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use simulate_price_fixed_methods::SIMULATE_PRICE_FIXED_GUEST_ELF;

pub fn simulate_price(
    de_seasonalised_detrended_log_base_fee: &DVector<FixedPoint>,
) -> (Receipt, (DVector<f64>, DMatrix<f64>, Vec<f64>)) {
    let env = ExecutorEnv::builder()
        // Send a & b to the guest
        .write(de_seasonalised_detrended_log_base_fee)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let prove_info = prover.prove(env, SIMULATE_PRICE_FIXED_GUEST_ELF).unwrap();

    let receipt = prove_info.receipt;
    let res: (DVector<f64>, DMatrix<f64>, Vec<f64>) = receipt.journal.decode().unwrap();

    (receipt, res)
}
