// use add_twap_7d_floating_methods::ADD_TWAP_7D_FLOATING_GUEST_ID;
// use remove_seasonality_floating_methods::REMOVE_SEASONALITY_FLOATING_GUEST_ID;
// use reserve_price_floating_methods::RESERVE_PRICE_FLOATING_GUEST_ID;
// use simulate_price_floating_methods::SIMULATE_PRICE_FLOATING_GUEST_ID;

// use reserve_price_floating_core::ReservePriceFloatingInput;

// use reserve_price_floating_composition_core::ReservePriceFloatingCompositionInput;
use reserve_price_composition_verify_simulated_price_floating_core::ReservePriceCompositionInput;
use risc0_zkvm::{guest::env, serde};
use simulate_price_verify_position_floating_methods::SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ID;
use simulate_price_verify_position_floating_core::SimulatePriceVerifyPositionInput;

fn main() {
    let data: ReservePriceCompositionInput = env::read();

    // fn main() {
    //     let data: SimulatePriceVerifyPositionInput = env::read();

    //     let (is_saddle_point, simulated_prices) = simulate_price_verify_position(
    //         &data.positions,
    //         &data.pt,
    //         &data.pt_1,
    //         data.gradient_tolerance,
    //         &data.de_seasonalised_detrended_log_base_fee,
    //         n_periods,
    //         num_paths,
    //     );

    //     assert!(is_saddle_point);

    //     // (SimulatePriceVerifyPositionInput, DMatrix<f64>)
    //     env::commit(&(data, simulated_prices));
    // }

    // pub struct SimulatePriceVerifyPositionInput {
    //     pub positions: Vec<f64>,
    //     pub pt: DVector<f64>,
    //     pub pt_1: DVector<f64>,
    //     pub gradient_tolerance: f64,
    //     pub de_seasonalised_detrended_log_base_fee: DVector<f64>,
    //     pub n_periods: usize,
    //     pub num_paths: usize,
    // }

    let simulate_price_verify_position_input = SimulatePriceVerifyPositionInput {
        positions: data.positions,
        pt: data.pt,
        pt_1: data.pt_1,
        gradient_tolerance: data.gradient_tolerance,
        de_seasonalised_detrended_log_base_fee: data.de_seasonalised_detrended_log_base_fee,
        n_periods: data.n_periods,
        num_paths: data.num_paths,
    };

    env::verify(
        SIMULATE_PRICE_VERIFY_POSITION_FLOATING_GUEST_ID,
        &serde::to_vec(&(
            simulate_price_verify_position_input,
            data.de_seasonalized_detrended_simulated_prices,
        ))
        .unwrap(),
    )
    .unwrap();

    // // TODO: to include inputs as public input and check it
    // // slope,intercept,de_seasonalised_detrended_log_base_fee,season_param,

    // env::verify(
    //     REMOVE_SEASONALITY_FLOATING_GUEST_ID,
    //     &serde::to_vec(&(
    //         data.slope,
    //         data.intercept,
    //         data.de_seasonalised_detrended_log_base_fee.clone(), // TODO: check if we can avoid cloning
    //         data.season_param.clone(),
    //     ))
    //     .unwrap(),
    // )
    // .unwrap();

    // // env::commit(&(de_seasonalised_detrended_log_base_fee, simulated_prices, params));
    // env::verify(
    //     SIMULATE_PRICE_FLOATING_GUEST_ID,
    //     &serde::to_vec(&(
    //         data.de_seasonalised_detrended_log_base_fee.clone(), // TODO: check if we can avoid cloning
    //         data.de_seasonalized_detrended_simulated_prices.clone(), // TODO: check if we can avoid cloning
    //         // data.season_param.clone(),
    //     ))
    //     .unwrap(),
    // )
    // .unwrap();

    // // env::commit(&(input_data, twap));
    // env::verify(
    //     ADD_TWAP_7D_FLOATING_GUEST_ID,
    //     &serde::to_vec(&(data.inputs.clone(), data.twap_7d.clone())).unwrap(),
    // )
    // .unwrap();

    // // (ReservePriceFloatingInput, f64)
    // let reserve_price_input = ReservePriceFloatingInput {
    //     period_start_timestamp: data.inputs[0].0,
    //     period_end_timestamp: data.inputs[data.inputs.len() - 1].0,
    //     season_param: data.season_param,
    //     de_seasonalized_detrended_simulated_prices: data.de_seasonalized_detrended_simulated_prices,
    //     twap_7d: data.twap_7d,
    //     slope: data.slope,
    //     intercept: data.intercept,
    //     log_base_fee_len: data.inputs.len(),
    // };
    // env::verify(
    //     RESERVE_PRICE_FLOATING_GUEST_ID,
    //     &serde::to_vec(&(reserve_price_input, data.reserve_price)).unwrap(),
    // )
    // .unwrap();
}
