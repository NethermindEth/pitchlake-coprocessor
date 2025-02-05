use add_twap_7d::add_twap_7d;
use add_twap_7d_methods::ADD_TWAP_7D_GUEST_ID;
use benchmark::hex_string_to_f64;
use db_access::queries::get_block_headers_by_block_range;
use db_access::DbConnection;
use dotenv::dotenv;
use eyre::Result;
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

    let (receipt, res) = add_twap_7d(&reserve_price_inputs);

    println!("res: {:?}", res);

    receipt.verify(ADD_TWAP_7D_GUEST_ID).unwrap();

    Ok(())
}
