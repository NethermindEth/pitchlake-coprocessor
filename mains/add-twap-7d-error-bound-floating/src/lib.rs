use add_twap_7d_error_bound_floating_methods::ADD_TWAP_7D_ERROR_BOUND_FLOATING_GUEST_ELF;
use core::AddTwap7dErrorBoundFloatingInput;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::thread;
use std::time::Duration;

pub fn add_twap_7d_error_bound(
    input: AddTwap7dErrorBoundFloatingInput,
) -> (Receipt, AddTwap7dErrorBoundFloatingInput) {
    let prover = default_prover();

    const MAX_RETRIES: u32 = 10;
    const INITIAL_DELAY_MS: u64 = 5000;

    let mut last_error = None;
    for attempt in 1..=MAX_RETRIES {
        let env = ExecutorEnv::builder()
            .write(&input)
            .unwrap()
            .build()
            .unwrap();

        match prover.prove(env, ADD_TWAP_7D_ERROR_BOUND_FLOATING_GUEST_ELF) {
            Ok(prove_info) => {
                let receipt = prove_info.receipt;
                let res: AddTwap7dErrorBoundFloatingInput = receipt.journal.decode().unwrap();
                return (receipt, res);
            }
            Err(e) => {
                let error_msg = e.to_string();
                let is_retryable = error_msg.contains("dns error")
                    || error_msg.contains("client error")
                    || error_msg.contains("error sending request")
                    || error_msg.contains("Connection refused")
                    || error_msg.contains("timeout");

                if !is_retryable || attempt == MAX_RETRIES {
                    last_error = Some(e);
                    break;
                }

                let delay = INITIAL_DELAY_MS * 2u64.pow(attempt - 1);
                thread::sleep(Duration::from_millis(delay));
            }
        }
    }

    panic!(
        "add_twap_7d_error_bound: Failed after {} attempts. Last error: {:?}",
        MAX_RETRIES,
        last_error.unwrap()
    );
}
