use add_twap_7d_methods::ADD_TWAP_7D_GUEST_ELF;
use nalgebra::{DMatrix, DVector};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub fn add_twap_7d(data: &Vec<(i64, f64)>) -> (Receipt, (Vec<(i64, f64)>, Vec<f64>)) {
    let env = ExecutorEnv::builder()
        // Send a & b to the guest
        .write(data)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let prove_info = prover.prove(env, ADD_TWAP_7D_GUEST_ELF).unwrap();

    let receipt = prove_info.receipt;
    let res: (Vec<(i64, f64)>, Vec<f64>) = receipt.journal.decode().unwrap();

    (receipt, res)
}
