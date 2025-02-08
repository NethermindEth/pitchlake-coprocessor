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
use tokio::{self};

use reserve_price_floating_composition_methods::{
    RESERVE_PRICE_FLOATING_COMPOSITION_GUEST_ELF, RESERVE_PRICE_FLOATING_COMPOSITION_GUEST_ID,
};

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
        simulate_price(&remove_seasonality_result.2); // use input from remove_seasonality_result due to math inconsistency
    let (add_twap_7d_receipt, _add_twap_7d_result) = add_twap_7d(&input_data);
    let reserve_price_input = ReservePriceFloatingInput {
        period_start_timestamp: input_data[0].0,
        period_end_timestamp: input_data[input_data.len() - 1].0,
        season_param: remove_seasonality_result.3.clone(),
        de_seasonalized_detrended_simulated_prices: simulate_price_result.1.clone(),
        twap_7d: all_inputs.twap_7d.clone(),
        slope: all_inputs.slope,
        intercept: all_inputs.intercept,
        log_base_fee_len: input_data.len(),
    };
    let (calculate_reserve_price_receipt, calculate_reserve_price_result) =
        reserve_price(reserve_price_input);

    // once we have all the receipts,
    // we can call the method to combine all the receipts above

    // let input = ReservePriceFloatingCompositionInput {
    //     inputs: input_data,
    //     season_param: all_inputs.season_param,
    //     de_seasonalised_detrended_log_base_fee: all_inputs.de_seasonalised_detrended_log_base_fee,
    //     de_seasonalized_detrended_simulated_prices: all_inputs
    //         .de_seasonalized_detrended_simulated_prices,
    //     twap_7d: all_inputs.twap_7d,
    //     slope: all_inputs.slope,
    //     intercept: all_inputs.intercept,
    //     reserve_price: all_inputs.reserve_price,
    // };
    let input = ReservePriceFloatingCompositionInput {
        inputs: input_data,
        season_param: remove_seasonality_result.3,
        de_seasonalised_detrended_log_base_fee: remove_seasonality_result.2,
        de_seasonalized_detrended_simulated_prices: simulate_price_result.1,
        twap_7d: all_inputs.twap_7d, // same result between host and guest
        slope: all_inputs.slope,     // same result between host and guest
        intercept: all_inputs.intercept, // same result between host and guest
        reserve_price: calculate_reserve_price_result.1,
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

    let prove_info = default_prover()
        .prove(env, RESERVE_PRICE_FLOATING_COMPOSITION_GUEST_ELF)
        .unwrap();

    let receipt = prove_info.receipt;
    receipt
        .verify(RESERVE_PRICE_FLOATING_COMPOSITION_GUEST_ID)
        .unwrap();

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
