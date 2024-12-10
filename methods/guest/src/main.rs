use risc0_zkvm::guest::env;
use db_models::BlockHeader;

mod pricing_data;


fn main() {
    let blocks: Vec<BlockHeader> = env::read();

    let volatility = pricing_data::volatility::calculate_volatility(blocks.clone());
    let twap = pricing_data::twap::calculate_twap(blocks.clone());
    let reserve_price = pricing_data::reserve_price::calculate_reserve_price(blocks.clone());

    env::commit(&(volatility.ok(), twap.ok(), reserve_price.ok()));
}
