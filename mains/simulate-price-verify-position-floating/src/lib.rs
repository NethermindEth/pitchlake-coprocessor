use core::SimulatePriceVerifyPositionInput;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use simulate_price_verify_position_floating_methods::SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ELF;
use std::thread;
use std::time::Duration;

pub fn simulate_price_verify_position(
    input: SimulatePriceVerifyPositionInput,
) -> (Receipt, SimulatePriceVerifyPositionInput) {
    let prover = default_prover();

    const MAX_RETRIES: u32 = 10;
    const INITIAL_DELAY_MS: u64 = 5000;

    let mut last_error = None;
    for attempt in 1..=MAX_RETRIES {
        eprintln!(
            "simulate_price_verify_position: Proof generation attempt {}/{}",
            attempt, MAX_RETRIES
        );

        let env = ExecutorEnv::builder()
            .write(&input)
            .unwrap()
            .build()
            .unwrap();

        match prover.prove(env, SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ELF) {
            Ok(prove_info) => {
                let receipt = prove_info.receipt;
                let res: SimulatePriceVerifyPositionInput = receipt.journal.decode().unwrap();
                eprintln!(
                    "simulate_price_verify_position: Proof generation succeeded on attempt {}",
                    attempt
                );
                return (receipt, res);
            }
            Err(e) => {
                eprintln!(
                    "simulate_price_verify_position: Attempt {}/{} failed: {}",
                    attempt, MAX_RETRIES, e
                );

                last_error = Some(e);

                if attempt == MAX_RETRIES {
                    // Final attempt - fail
                    break;
                }

                // Exponential backoff: 5s, 10s, 20s, 40s, etc.
                let delay = INITIAL_DELAY_MS * 2u64.pow(attempt - 1);
                eprintln!("simulate_price_verify_position: Retrying in {}ms...", delay);
                thread::sleep(Duration::from_millis(delay));
            }
        }
    }

    panic!(
        "simulate_price_verify_position: Failed after {} attempts. Last error: {:?}",
        MAX_RETRIES,
        last_error.unwrap()
    );
}
