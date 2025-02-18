use add_twap_7d_floating_methods::ADD_TWAP_7D_FLOATING_GUEST_ID;
use remove_seasonality_floating_methods::REMOVE_SEASONALITY_FLOATING_GUEST_ID;
use reserve_price_floating_methods::RESERVE_PRICE_FLOATING_GUEST_ID;
use simulate_price_floating_methods::SIMULATE_PRICE_FLOATING_GUEST_ID;

use reserve_price_floating_core::ReservePriceFloatingInput;

use reserve_price_floating_composition_core::ReservePriceFloatingCompositionInput;
use risc0_zkvm::{guest::env, serde};

fn main() {
    let data: ReservePriceFloatingCompositionInput = env::read();
    // TODO: to include inputs as public input and check it
    // slope,intercept,de_seasonalised_detrended_log_base_fee,season_param,

    env::verify(
        REMOVE_SEASONALITY_FLOATING_GUEST_ID,
        &serde::to_vec(&(
            data.slope,
            data.intercept,
            data.de_seasonalised_detrended_log_base_fee.clone(), // TODO: check if we can avoid cloning
            data.season_param.clone(),
        ))
        .unwrap(),
    )
    .unwrap();

    // env::commit(&(de_seasonalised_detrended_log_base_fee, simulated_prices, params));
    env::verify(
        SIMULATE_PRICE_FLOATING_GUEST_ID,
        &serde::to_vec(&(
            data.de_seasonalised_detrended_log_base_fee.clone(), // TODO: check if we can avoid cloning
            data.de_seasonalized_detrended_simulated_prices.clone(), // TODO: check if we can avoid cloning
            // data.season_param.clone(),
        ))
        .unwrap(),
    )
    .unwrap();

    // env::commit(&(input_data, twap));
    env::verify(
        ADD_TWAP_7D_FLOATING_GUEST_ID,
        &serde::to_vec(&(data.inputs.clone(), data.twap_7d.clone())).unwrap(),
    )
    .unwrap();

    // (ReservePriceFloatingInput, f64)
    let reserve_price_input = ReservePriceFloatingInput {
        period_start_timestamp: data.inputs[0].0,
        period_end_timestamp: data.inputs[data.inputs.len() - 1].0,
        season_param: data.season_param,
        de_seasonalized_detrended_simulated_prices: data.de_seasonalized_detrended_simulated_prices,
        twap_7d: data.twap_7d,
        slope: data.slope,
        intercept: data.intercept,
        log_base_fee_len: data.inputs.len(),
    };
    env::verify(
        RESERVE_PRICE_FLOATING_GUEST_ID,
        &serde::to_vec(&(reserve_price_input, data.reserve_price)).unwrap(),
    )
    .unwrap();
}
