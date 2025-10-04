use core::MaxReturnInput;
use max_return_floating_methods::MAX_RETURN_FLOATING_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::thread;
use std::time::Duration;

pub fn max_return(input: MaxReturnInput) -> (Receipt, (MaxReturnInput, f64)) {
    eprintln!(
        "max_return: Received {} data points for max return calculation",
        input.data.len()
    );

    let prover = default_prover();

    const MAX_RETRIES: u32 = 10;
    const INITIAL_DELAY_MS: u64 = 5000;

    let mut last_error = None;
    for attempt in 1..=MAX_RETRIES {
        eprintln!(
            "max_return: Proof generation attempt {}/{}",
            attempt, MAX_RETRIES
        );

        let env = ExecutorEnv::builder()
            .write(&input)
            .unwrap()
            .build()
            .unwrap();

        match prover.prove(env, MAX_RETURN_FLOATING_GUEST_ELF) {
            Ok(prove_info) => {
                let receipt = prove_info.receipt;
                let (input, max_return): (MaxReturnInput, f64) = receipt.journal.decode().unwrap();
                eprintln!(
                    "max_return: Proof generation succeeded on attempt {}",
                    attempt
                );
                return (receipt, (input, max_return));
            }
            Err(e) => {
                eprintln!(
                    "max_return: Attempt {}/{} failed: {}",
                    attempt, MAX_RETRIES, e
                );

                last_error = Some(e);

                if attempt == MAX_RETRIES {
                    // Final attempt - fail
                    break;
                }

                // Exponential backoff: 5s, 10s, 20s, 40s, etc.
                let delay = INITIAL_DELAY_MS * 2u64.pow(attempt - 1);
                eprintln!("max_return: Retrying in {}ms...", delay);
                thread::sleep(Duration::from_millis(delay));
            }
        }
    }

    // All retries failed
    panic!(
        "max_return: Failed after {} attempts. Last error: {:?}",
        MAX_RETRIES,
        last_error.unwrap()
    );
}
