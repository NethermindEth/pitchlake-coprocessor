// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use core::VolatilityInputsFixedPointSimba;
use db_access::queries::get_block_headers_by_block_range;
use db_access::DbConnection;
use dotenv::dotenv;
use eyre::Result;
use methods::PRICING_CALCULATOR_ELF;
use methods_reserve_price::GUEST_RESERVE_PRICE_ELF;
use methods_twap::GUEST_TWAP_ELF;
use methods_volatility::GUEST_VOLATILITY_ELF;
use num_traits::Zero;
use risc0_zkvm::{default_prover, ExecutorEnv};
use simba::scalar::{FixedI48F16, RealField};
use tokio::task;

fn hex_string_to_f64(hex_str: &String) -> Result<f64> {
    let stripped = hex_str.trim_start_matches("0x");
    u128::from_str_radix(stripped, 16)
        .map(|value| value as f64)
        .map_err(|e| eyre::eyre!("Error converting hex string '{}' to f64: {}", hex_str, e))
}

// todo: accept generic fixed point type with trait bound
fn natural_log(x: FixedI48F16) -> Result<FixedI48F16> {
    if x <= FixedI48F16::zero() {
        return Err(eyre::eyre!("Cannot take logarithm of non-positive number"));
    }
    let mut power = 0i32;
    let two = FixedI48F16::from_num(2);
    let one = FixedI48F16::from_num(1);
    let mut val = x;
    while val >= two {
        val = val / two;
        power += 1;
    }
    while val < one {
        val = val * two;
        power -= 1;
    }
    let base_ln = FixedI48F16::ln_2() * FixedI48F16::from_num(power);
    let frac = val - one;
    let frac_contribution = frac * FixedI48F16::ln_2();
    Ok(base_ln + frac_contribution)
}

async fn run_host(
    start_block: i64,
    end_block: i64,
) -> Result<(Option<f64>, Option<f64>, Option<f64>), sqlx::Error> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let db = DbConnection::new().await?;
    let block_headers = get_block_headers_by_block_range(&db.pool, start_block, end_block).await?;

    let prove_info = task::spawn_blocking(move || {
        let env = ExecutorEnv::builder()
            .write(&block_headers)
            .unwrap()
            .build()
            .unwrap();
        let prover = default_prover();
        prover.prove(env, PRICING_CALCULATOR_ELF).unwrap()
    })
    .await
    .unwrap();

    let receipt = prove_info.receipt;

    let (volatility, twap, reserve_price): (Option<f64>, Option<f64>, Option<f64>) =
        receipt.journal.decode().unwrap();

    println!("HOST");
    println!("Volatility: {:?}", volatility);
    println!("TWAP: {:?}", twap);
    println!("Reserve Price: {:?}", reserve_price);

    Ok((volatility, twap, reserve_price))
}

async fn run_host_reserve_price(start_block: i64, end_block: i64) -> Result<f64, sqlx::Error> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let db = DbConnection::new().await?;
    let block_headers = get_block_headers_by_block_range(&db.pool, start_block, end_block).await?;

    let base_fee_per_gases_hex: Vec<Option<String>> = block_headers
        .iter()
        .map(|header| header.base_fee_per_gas.clone())
        .collect();

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

    let prove_info = task::spawn_blocking(move || {
        let env = ExecutorEnv::builder()
            .write(&reserve_price_inputs)
            .unwrap()
            .build()
            .unwrap();
        let prover = default_prover();
        prover.prove(env, GUEST_RESERVE_PRICE_ELF).unwrap()
    })
    .await
    .unwrap();

    let receipt = prove_info.receipt;
    let reserve_price: f64 = receipt.journal.decode().unwrap();
    Ok(reserve_price)
}

async fn run_host_twap(start_block: i64, end_block: i64) -> Result<FixedI48F16, sqlx::Error> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let db = DbConnection::new().await?;
    let block_headers = get_block_headers_by_block_range(&db.pool, start_block, end_block).await?;

    let base_fee_per_gases_hex: Vec<Option<String>> = block_headers
        .iter()
        .map(|header| header.base_fee_per_gas.clone())
        .collect();
    let base_fee_per_gases: Vec<FixedI48F16> = base_fee_per_gases_hex
        .iter()
        .map(|hexes| {
            if let Some(hex) = hexes {
                // return hex_string_to_f64(hex).unwrap();
                return FixedI48F16::from_num(hex_string_to_f64(hex).unwrap());
            } else {
                return FixedI48F16::zero();
            }
        })
        .collect();

    let prove_info = task::spawn_blocking(move || {
        let env = ExecutorEnv::builder()
            .write(&base_fee_per_gases)
            .unwrap()
            .build()
            .unwrap();
        let prover = default_prover();
        prover.prove(env, GUEST_TWAP_ELF).unwrap()
    })
    .await
    .unwrap();

    let receipt = prove_info.receipt;

    let twap: FixedI48F16 = receipt.journal.decode().unwrap();

    Ok(twap)
}

async fn run_host_volatility_fixed_point(
    start_block: i64,
    end_block: i64,
) -> Result<FixedI48F16, sqlx::Error> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let db = DbConnection::new().await?;
    let block_headers = get_block_headers_by_block_range(&db.pool, start_block, end_block).await?;
    let base_fee_per_gases_hex: Vec<Option<String>> = block_headers
        .iter()
        .map(|header| header.base_fee_per_gas.clone())
        .collect();
    let base_fee_per_gases: Vec<Option<FixedI48F16>> = base_fee_per_gases_hex
        .iter()
        .map(|hexes| {
            if let Some(hex) = hexes {
                return Some(FixedI48F16::from_num(hex_string_to_f64(hex).unwrap()));
            } else {
                return None;
            }
        })
        .collect();

    let mut results: Vec<FixedI48F16> = Vec::new();
    for i in 1..base_fee_per_gases.len() {
        if let (Some(ref basefee_current), Some(ref basefee_previous)) =
            (&base_fee_per_gases[i], &base_fee_per_gases[i - 1])
        {
            if basefee_previous.is_zero() {
                continue;
            }

            // Calculate log return and add it to the returns vector
            results.push(natural_log(*basefee_current / *basefee_previous).unwrap());
        }
    }

    let volatility_inputs = VolatilityInputsFixedPointSimba {
        base_fee_per_gases: base_fee_per_gases.clone(),
        ln_results: results.clone(),
    };

    let prove_info = task::spawn_blocking(move || {
        let env = ExecutorEnv::builder()
            .write(&volatility_inputs)
            .unwrap()
            .build()
            .unwrap();
        let prover = default_prover();
        prover.prove(env, GUEST_VOLATILITY_ELF).unwrap()
    })
    .await
    .unwrap();

    let receipt = prove_info.receipt;
    let volatility: FixedI48F16 = receipt.journal.decode().unwrap();
    Ok(volatility)
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // run_host(20000000, 20000170).await?;
    // run_host(20000000, 20000200).await?;
    // 3 months data
    // run_host(20000000, 20700000).await?;
    // run_host_volatility_fixed_point(20000000, 20002000).await?;
    // run_host_twap(20000000, 20002000).await?;
    run_host_reserve_price(20000000, 20000168).await?;
    Ok(())
}
