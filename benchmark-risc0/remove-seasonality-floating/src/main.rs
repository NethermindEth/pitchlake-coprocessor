use benchmark::tests::mock::get_first_period_data;
use eyre::Result;
use remove_seasonality_floating::remove_seasonality;
use remove_seasonality_floating_methods::REMOVE_SEASONALITY_FLOATING_GUEST_ID;

fn main() -> Result<(), String> {
    let reserve_price_inputs = get_first_period_data();

    let (receipt, res) = remove_seasonality(&reserve_price_inputs.iter().map(|x| x.1).collect());

    println!("res: {:?}", res);

    receipt
        .verify(REMOVE_SEASONALITY_FLOATING_GUEST_ID)
        .unwrap();

    Ok(())
}
