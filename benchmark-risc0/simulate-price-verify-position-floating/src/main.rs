use benchmark::{original::calculate_reserve_price, tests::mock::get_first_period_data};

fn main() {
    // get only first period of (timestamp avg_gas_fee)
    let data = get_first_period_data();
    // run rust code in host
    // ensure convergence in host
    let res = calculate_reserve_price(&data);
    println!("res: {:?}", res);
    // create input for guest
    // run guest
    // verify guest receipt
}
