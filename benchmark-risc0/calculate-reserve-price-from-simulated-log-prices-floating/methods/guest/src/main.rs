use benchmark::floating_point::calculated_reserve_price_from_simulated_log_prices;
use calculate_reserve_price_from_simulated_log_prices_floating_core::CalculateReservePriceFromSimulatedLogPricesInput;
use risc0_zkvm::guest::env;

fn main() {
    let data: CalculateReservePriceFromSimulatedLogPricesInput = env::read();
    let reserve_price = calculated_reserve_price_from_simulated_log_prices(
        &data.simulated_log_prices,
        &data.twap_7d,
        data.n_periods,
    ).unwrap();
    env::commit(&(data, reserve_price));
}
