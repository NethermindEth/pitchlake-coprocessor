use hashing_felts_methods::HASHING_FELTS_GUEST_ELF;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::thread;
use std::time::Duration;

use core::{HashingFeltInput, HashingFeltOutput};

pub fn hash_felts(input: HashingFeltInput) -> (Receipt, HashingFeltOutput) {
    let prover = default_prover();

    const MAX_RETRIES: u32 = 10;
    const INITIAL_DELAY_MS: u64 = 5000;

    let mut last_error = None;
    for attempt in 1..=MAX_RETRIES {
        eprintln!(
            "hash_felts: Proof generation attempt {}/{}",
            attempt, MAX_RETRIES
        );

        let env = ExecutorEnv::builder()
            .write(&input)
            .unwrap()
            .build()
            .unwrap();

        match prover.prove(env, HASHING_FELTS_GUEST_ELF) {
            Ok(prove_info) => {
                let receipt = prove_info.receipt;
                let res: HashingFeltOutput = receipt.journal.decode().unwrap();
                eprintln!(
                    "hash_felts: Proof generation succeeded on attempt {}",
                    attempt
                );
                return (receipt, res);
            }
            Err(e) => {
                eprintln!(
                    "hash_felts: Attempt {}/{} failed: {}",
                    attempt, MAX_RETRIES, e
                );

                last_error = Some(e);

                if attempt == MAX_RETRIES {
                    // Final attempt - fail
                    break;
                }

                // Exponential backoff: 5s, 10s, 20s, 40s, etc.
                let delay = INITIAL_DELAY_MS * 2u64.pow(attempt - 1);
                eprintln!("hash_felts: Retrying in {}ms...", delay);
                thread::sleep(Duration::from_millis(delay));
            }
        }
    }

    // All retries failed
    panic!(
        "hash_felts: Failed after {} attempts. Last error: {:?}",
        MAX_RETRIES,
        last_error.unwrap()
    );
}
