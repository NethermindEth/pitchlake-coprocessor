use add_twap_7d_floating::add_twap_7d;
use add_twap_7d_floating_methods::ADD_TWAP_7D_FLOATING_GUEST_ID;
use benchmark::tests::mock::get_first_period_data;
use eyre::Result;

fn main() {
    let reserve_price_inputs = get_first_period_data();

    let (receipt, res) = add_twap_7d(&reserve_price_inputs.iter().map(|x| x.1).collect());

    println!("res: {:?}", res);

    receipt.verify(ADD_TWAP_7D_FLOATING_GUEST_ID).unwrap();
}
