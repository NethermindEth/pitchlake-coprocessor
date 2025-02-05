use benchmark::{floating_point::calculate_remove_seasonality, hex_string_to_f64};
use db_access::queries::get_block_headers_by_block_range;
use db_access::DbConnection;
use dotenv::dotenv;
use eyre::Result;
use simulate_price_floating::simulate_price;
use simulate_price_floating_methods::SIMULATE_PRICE_FLOATING_GUEST_ID;
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
    let (_slope, _intercept, de_seasonalised_detrended_log_base_fee, _season_param) =
        calculate_remove_seasonality(&reserve_price_inputs).unwrap();

    // pass calculated result in host to guest
    let (receipt, res) = simulate_price(&de_seasonalised_detrended_log_base_fee);
    println!("res: {:?}", res);

    receipt.verify(SIMULATE_PRICE_FLOATING_GUEST_ID).unwrap();

    Ok(())
}
