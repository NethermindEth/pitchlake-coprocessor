use nalgebra::DVector;
use remove_seasonality_floating_methods::REMOVE_SEASONALITY_FLOATING_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub fn remove_seasonality(
    data: &Vec<f64>,
) -> (Receipt, (f64, f64, DVector<f64>, DVector<f64>)) {
    let env = ExecutorEnv::builder()
        // Send a & b to the guest
        .write(data)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let prove_info = prover
        .prove(env, REMOVE_SEASONALITY_FLOATING_GUEST_ELF)
        .unwrap();

    let receipt = prove_info.receipt;
    let res: (f64, f64, DVector<f64>, DVector<f64>) = receipt.journal.decode().unwrap();

    (receipt, res)
}
