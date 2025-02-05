use benchmark::floating_point::{add_twap_7d, calculate_remove_seasonality, simulate_price};
use benchmark::hex_string_to_f64;
use db_access::queries::get_block_headers_by_block_range;
use db_access::DbConnection;
use dotenv::dotenv;
use eyre::Result;
use reserve_price_floating::reserve_price;
use reserve_price_floating_core::ReservePriceFloatingInput;
use reserve_price_floating_methods::RESERVE_PRICE_FLOATING_GUEST_ID;
use tokio::main;

#[main]
async fn main() -> Result<(), String> {
    let start_block = 20000000;
    // let end_block = 20002000;
    let end_block = 20002160;

    dotenv().ok();
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
    // calculate this in host
    let (slope, intercept, de_seasonalised_detrended_log_base_fee, season_param) =
        calculate_remove_seasonality(&reserve_price_inputs).unwrap();

    let (simulated_prices, _params) = simulate_price(&de_seasonalised_detrended_log_base_fee);

    let twap = add_twap_7d(&reserve_price_inputs).unwrap();

    let input = ReservePriceFloatingInput {
        period_start_timestamp: reserve_price_inputs[0].0,
        period_end_timestamp: reserve_price_inputs[reserve_price_inputs.len() - 1].0,
        season_param,
        de_seasonalized_detrended_simulated_prices: simulated_prices,
        twap_7d: twap,
        slope,
        intercept,
        log_base_fee_len: reserve_price_inputs.len(),
    };

    let (receipt, res) = reserve_price(input);

    println!("res: {:?}", res);
    receipt.verify(RESERVE_PRICE_FLOATING_GUEST_ID).unwrap();

    Ok(())
}
