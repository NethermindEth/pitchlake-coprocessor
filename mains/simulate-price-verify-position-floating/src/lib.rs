use core::SimulatePriceVerifyPositionInput;
use std::thread;
use std::time::Duration;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use simulate_price_verify_position_floating_methods::SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ELF;

pub fn simulate_price_verify_position(
    input: SimulatePriceVerifyPositionInput,
) -> (Receipt, SimulatePriceVerifyPositionInput) {
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    let prover = default_prover();

    const MAX_RETRIES: u32 = 5;
    const INITIAL_DELAY_MS: u64 = 2000;

    let mut last_error = None;
    for attempt in 1..=MAX_RETRIES {
        match prover.prove(env.clone(), SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ELF) {
            Ok(prove_info) => {
                let receipt = prove_info.receipt;
                let res: SimulatePriceVerifyPositionInput = receipt.journal.decode().unwrap();
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

    panic!("simulate_price_verify_position: Failed after {} attempts. Last error: {:?}", MAX_RETRIES, last_error.unwrap());
}
