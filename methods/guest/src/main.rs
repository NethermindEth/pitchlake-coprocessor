use risc0_zkvm::guest::env;
use eth_rlp_types::block_header::BlockHeader;

mod pricing_data;


fn main() {
    let blocks: Vec<BlockHeader> = env::read();

    let volatility = pricing_data::volatility::calculate_volatility(blocks.clone());
    let twap = pricing_data::twap::calculate_twap(blocks.clone());

    env::commit(&(volatility.ok(), twap.ok()));
}
