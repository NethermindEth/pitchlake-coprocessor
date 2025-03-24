use common::tests::mock::get_max_return_input_data;
use core::MaxReturnInput;
use max_return_floating::max_return;
use max_return_floating_methods::MAX_RETURN_FLOATING_GUEST_ID;

fn main() {
    let data = get_max_return_input_data();
    let input = MaxReturnInput {
        data: data.iter().map(|x| x.1).collect::<Vec<f64>>(),
    };
    let (receipt, res) = max_return(input);

    receipt.verify(MAX_RETURN_FLOATING_GUEST_ID).unwrap();
    println!("max_return: {:?}", res.1);
}
