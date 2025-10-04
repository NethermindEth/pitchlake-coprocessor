use calculate_pt_pt1_error_bound_floating_methods::CALCULATE_PT_PT1_ERROR_BOUND_FLOATING_GUEST_ELF;
use core::CalculatePtPt1ErrorBoundFloatingInput;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::thread;
use std::time::Duration;

pub fn calculate_pt_pt1_error_bound_floating(
    input: CalculatePtPt1ErrorBoundFloatingInput,
) -> (Receipt, CalculatePtPt1ErrorBoundFloatingInput) {
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

        match prover.prove(env, CALCULATE_PT_PT1_ERROR_BOUND_FLOATING_GUEST_ELF) {
            Ok(prove_info) => {
                let receipt = prove_info.receipt;
                let res: CalculatePtPt1ErrorBoundFloatingInput = receipt.journal.decode().unwrap();
                return (receipt, res);
            }
            Err(e) => {
                eprintln!(
                    "calculate_pt_pt1_error_bound_floating: Attempt {}/{} failed: {}",
                    attempt, MAX_RETRIES, e
                );

                last_error = Some(e);

                if attempt == MAX_RETRIES {
                    // Final attempt - fail
                    break;
                }

                // Exponential backoff: 5s, 10s, 20s, 40s, etc.
                let delay = INITIAL_DELAY_MS * 2u64.pow(attempt - 1);
                eprintln!(
                    "calculate_pt_pt1_error_bound_floating: Retrying in {}ms...",
                    delay
                );
                thread::sleep(Duration::from_millis(delay));
            }
        }
    }

    panic!(
        "calculate_pt_pt1_error_bound_floating: Failed after {} attempts. Last error: {:?}",
        MAX_RETRIES,
        last_error.unwrap()
    );
}
