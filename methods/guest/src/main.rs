use risc0_zkvm::guest::env;
use eth_rlp_types::BlockHeader;

mod pricing_data;


fn main() {
    let blocks: Vec<BlockHeader> = env::read();

    // let volatility = pricing_data::volatility::calculate_volatility(blocks.clone());
    // let twap = pricing_data::twap::calculate_twap(blocks.clone());
    let reserve_price = pricing_data::reserve_price::calculate_reserve_price(blocks.clone());
    // println!("Volatility: {:?}", volatility);
    // println!("TWAP: {:?}", twap);
    println!("Reserve Price: {:?}", reserve_price);

    // env::commit(&(volatility.ok(), twap.ok(), reserve_price.ok()));
    env::commit(&(None::<Option::<f64>>, None::<Option::<f64>>, reserve_price.ok()));
}
