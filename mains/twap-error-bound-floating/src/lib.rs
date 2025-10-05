use core::TwapErrorBoundInput;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use twap_error_bound_floating_methods::TWAP_ERROR_BOUND_FLOATING_GUEST_ELF;

pub fn calculate_twap(input: TwapErrorBoundInput) -> (Receipt, TwapErrorBoundInput) {
    const MAX_RETRIES: u32 = 10;
    const INITIAL_DELAY_MS: u64 = 5000;
    const PROOF_TIMEOUT_SECS: u64 = 1800; // 30 minutes timeout per attempt

    let mut last_error = None;
    for attempt in 1..=MAX_RETRIES {
        eprintln!(
            "calculate_twap: Proof generation attempt {}/{}",
            attempt, MAX_RETRIES
        );

        // Create a new prover instance per attempt to avoid state issues
        let prover = default_prover();

        let env = ExecutorEnv::builder()
            .write(&input)
            .unwrap()
            .build()
            .unwrap();

        // Spawn proof generation in a separate thread with timeout
        let (tx, rx) = mpsc::channel();
        let input_clone = input.clone();

        thread::spawn(move || {
            let result = prover.prove(env, TWAP_ERROR_BOUND_FLOATING_GUEST_ELF);
            let _ = tx.send(result);
        });

        // Wait for result with timeout
        match rx.recv_timeout(Duration::from_secs(PROOF_TIMEOUT_SECS)) {
            Ok(Ok(prove_info)) => {
                let receipt = prove_info.receipt;
                let res: TwapErrorBoundInput = receipt.journal.decode().unwrap();
                eprintln!(
                    "calculate_twap: Proof generation succeeded on attempt {}",
                    attempt
                );
                return (receipt, res);
            }
            Ok(Err(e)) => {
                eprintln!(
                    "calculate_twap: Attempt {}/{} failed: {}",
                    attempt, MAX_RETRIES, e
                );
                last_error = Some(format!("{}", e));
            }
            Err(_) => {
                eprintln!(
                    "calculate_twap: Attempt {}/{} timed out after {}s",
                    attempt, MAX_RETRIES, PROOF_TIMEOUT_SECS
                );
                last_error = Some(format!("Timeout after {}s", PROOF_TIMEOUT_SECS));
            }
        }

        if attempt == MAX_RETRIES {
            // Final attempt - fail
            break;
        }

        // Exponential backoff: 5s, 10s, 20s, 40s, etc.
        let delay = INITIAL_DELAY_MS * 2u64.pow(attempt - 1);
        eprintln!("calculate_twap: Retrying in {}ms...", delay);
        thread::sleep(Duration::from_millis(delay));
    }

    panic!(
        "calculate_twap: Failed after {} attempts. Last error: {}",
        MAX_RETRIES,
        last_error.unwrap()
    );
}
