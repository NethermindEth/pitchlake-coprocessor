use reserve_price_floating_core::ReservePriceFloatingInput;
use reserve_price_floating_methods::RESERVE_PRICE_FLOATING_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub fn reserve_price(
    data: ReservePriceFloatingInput,
) -> (Receipt, (ReservePriceFloatingInput, f64)) {
    let env = ExecutorEnv::builder()
        .write(&data)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Produce a receipt by proving the specified ELF binary.
    let prove_info = prover.prove(env, RESERVE_PRICE_FLOATING_GUEST_ELF).unwrap();

    let receipt = prove_info.receipt;
    let res: (ReservePriceFloatingInput, f64) = receipt.journal.decode().unwrap();

    (receipt, res)
}
