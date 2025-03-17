use remove_seasonality_error_bound_floating_core::RemoveSeasonalityErrorBoundFloatingInput;
use remove_seasonality_error_bound_floating_methods::REMOVE_SEASONALITY_ERROR_BOUND_FLOATING_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

pub fn remove_seasonality_error_bound(
    input: RemoveSeasonalityErrorBoundFloatingInput,
) -> (Receipt, RemoveSeasonalityErrorBoundFloatingInput) {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();
    let prove_info = prover
        .prove(env, REMOVE_SEASONALITY_ERROR_BOUND_FLOATING_GUEST_ELF)
        .unwrap();

    let receipt = prove_info.receipt;
    let res: RemoveSeasonalityErrorBoundFloatingInput = receipt.journal.decode().unwrap();

    (receipt, res)
}
