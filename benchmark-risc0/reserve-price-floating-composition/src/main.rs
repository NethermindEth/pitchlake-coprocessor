use add_twap_7d_floating::add_twap_7d;
use benchmark::floating_point::calculate_reserve_price_full;
use benchmark::hex_string_to_f64;
use db_access::queries::get_block_headers_by_block_range;
use db_access::DbConnection;
use dotenv::dotenv;
use eyre::Result;
use remove_seasonality_floating::remove_seasonality;
use reserve_price_floating::reserve_price;
use reserve_price_floating_composition_core::ReservePriceFloatingCompositionInput;
use reserve_price_floating_core::ReservePriceFloatingInput;
use risc0_zkvm::{default_prover, ExecutorEnv};
use simulate_price_floating::simulate_price;
use tokio;

use reserve_price_floating_composition_methods::RESERVE_PRICE_FLOATING_COMPOSITION_GUEST_ELF;

#[tokio::main]
async fn main() -> Result<(), String> {
    let start_block = 20000000;
    let end_block = 20002000;

    let input_data = get_input_data(start_block, end_block).await;

    // host calculation
    let all_inputs = calculate_reserve_price_full(&input_data);

    // guest calculation
    // all of these can be done in parallel
    let (remove_seasonality_receipt, remove_seasonality_result) = remove_seasonality(&input_data);
    let (simulate_price_receipt, simulate_price_result) =
        simulate_price(&all_inputs.de_seasonalised_detrended_log_base_fee);
    let (add_twap_7d_receipt, add_twap_7d_result) = add_twap_7d(&input_data);
    let reserve_price_input = ReservePriceFloatingInput {
        period_start_timestamp: input_data[0].0,
        period_end_timestamp: input_data[input_data.len() - 1].0,
        season_param: all_inputs.season_param.clone(),
        de_seasonalized_detrended_simulated_prices: all_inputs
            .de_seasonalized_detrended_simulated_prices
            .clone(),
        twap_7d: all_inputs.twap_7d.clone(),
        slope: all_inputs.slope,
        intercept: all_inputs.intercept,
        log_base_fee_len: input_data.len(),
    };
    let (calculate_reserve_price_receipt, calculate_reserve_price_result) =
        reserve_price(reserve_price_input);

    // once we have all the receipts,
    // we can call the method to combine all the receipts above

    let input = ReservePriceFloatingCompositionInput {
        inputs: input_data,
        season_param: all_inputs.season_param,
        de_seasonalised_detrended_log_base_fee: all_inputs.de_seasonalised_detrended_log_base_fee,
        de_seasonalized_detrended_simulated_prices: all_inputs
            .de_seasonalized_detrended_simulated_prices,
        twap_7d: all_inputs.twap_7d,
        slope: all_inputs.slope,
        intercept: all_inputs.intercept,
        reserve_price: all_inputs.reserve_price,
    };

    let env = ExecutorEnv::builder()
        .add_assumption(remove_seasonality_receipt)
        .add_assumption(simulate_price_receipt)
        .add_assumption(add_twap_7d_receipt)
        .add_assumption(calculate_reserve_price_receipt)
        .write(&input)
        .unwrap()
        .build()
        .unwrap();

    let receipt = default_prover()
        .prove(env, RESERVE_PRICE_FLOATING_COMPOSITION_GUEST_ELF)
        .unwrap();

    // // Anybody who receives the receipt for the exponentiation is assured both that:
    // // A) The modulus n included in the journal has a known factorization.
    // // B) The number c is the result of exponentiation of some known secret x ^ e mod n.
    // //
    // // These two statements are proven with a single receipt via composition.
    // receipt.verify(EXPONENTIATE_ID).unwrap();

    // // Decode the receipt to get (n, e, and c = x^e mod n).
    // let (n, e, c): (u64, u64, u64) = receipt.journal.decode().unwrap();

    Ok(())
}

async fn get_input_data(start_block: i64, end_block: i64) -> Vec<(i64, f64)> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let db = DbConnection::new().await.unwrap();
    let block_headers = get_block_headers_by_block_range(&db.pool, start_block, end_block)
        .await
        .unwrap();

    let mut reserve_price_inputs: Vec<(i64, f64)> = block_headers
        .iter()
        .map(|header| {
            let timestamp = i64::from_str_radix(
                header
                    .timestamp
                    .clone()
                    .unwrap()
                    .strip_prefix("0x")
                    .unwrap(),
                16,
            )
            .unwrap();

            let base_fee = hex_string_to_f64(&header.base_fee_per_gas.clone().unwrap()).unwrap();

            return (timestamp * 1000, base_fee);
        })
        .collect();

    reserve_price_inputs.sort_by(|a, b| a.0.cmp(&b.0));

    reserve_price_inputs
}
