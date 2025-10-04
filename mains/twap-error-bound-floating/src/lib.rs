use core::TwapErrorBoundInput;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::thread;
use std::time::Duration;
use twap_error_bound_floating_methods::TWAP_ERROR_BOUND_FLOATING_GUEST_ELF;

pub fn calculate_twap(input: TwapErrorBoundInput) -> (Receipt, TwapErrorBoundInput) {
    let prover = default_prover();

    const MAX_RETRIES: u32 = 5;
    const INITIAL_DELAY_MS: u64 = 2000;

    let mut last_error = None;
    for attempt in 1..=MAX_RETRIES {
        let env = ExecutorEnv::builder()
            .write(&input)
            .unwrap()
            .build()
            .unwrap();

        match prover.prove(env, TWAP_ERROR_BOUND_FLOATING_GUEST_ELF) {
            Ok(prove_info) => {
                let receipt = prove_info.receipt;
                let res: TwapErrorBoundInput = receipt.journal.decode().unwrap();
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
        "calculate_twap: Failed after {} attempts. Last error: {:?}",
        MAX_RETRIES,
        last_error.unwrap()
    );
}
