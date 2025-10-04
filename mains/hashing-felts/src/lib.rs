use hashing_felts_methods::HASHING_FELTS_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::thread;
use std::time::Duration;

use core::{HashingFeltInput, HashingFeltOutput};

pub fn hash_felts(input: HashingFeltInput) -> (Receipt, HashingFeltOutput) {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Retry logic for Bonsai API calls with exponential backoff
    const MAX_RETRIES: u32 = 5;
    const INITIAL_DELAY_MS: u64 = 2000;

    let mut last_error = None;
    for attempt in 1..=MAX_RETRIES {
        eprintln!("hash_felts: Proof generation attempt {}/{}", attempt, MAX_RETRIES);

        match prover.prove(env.clone(), HASHING_FELTS_GUEST_ELF) {
            Ok(prove_info) => {
                let receipt = prove_info.receipt;
                let res: HashingFeltOutput = receipt.journal.decode().unwrap();
                eprintln!("hash_felts: Proof generation succeeded on attempt {}", attempt);
                return (receipt, res);
            }
            Err(e) => {
                let error_msg = e.to_string();
                eprintln!("hash_felts: Attempt {}/{} failed: {}", attempt, MAX_RETRIES, error_msg);

                // Check if it's a retryable error (network issues)
                let is_retryable = error_msg.contains("dns error")
                    || error_msg.contains("client error")
                    || error_msg.contains("error sending request")
                    || error_msg.contains("Connection refused")
                    || error_msg.contains("timeout");

                if !is_retryable || attempt == MAX_RETRIES {
                    // Non-retryable error or final attempt - fail
                    last_error = Some(e);
                    break;
                }

                // Exponential backoff: 2s, 4s, 8s, 16s
                let delay = INITIAL_DELAY_MS * 2u64.pow(attempt - 1);
                eprintln!("hash_felts: Retrying in {}ms...", delay);
                thread::sleep(Duration::from_millis(delay));
            }
        }
    }

    // All retries failed
    panic!("hash_felts: Failed after {} attempts. Last error: {:?}", MAX_RETRIES, last_error.unwrap());
}
