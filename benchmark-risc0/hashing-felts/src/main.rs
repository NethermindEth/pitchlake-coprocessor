use common::{convert_felt_to_f64, tests::mock::get_5760_avg_base_fees_felt};
use hashing_felts::hash_felts;
use hashing_felts_core::HashingFeltInput;
use hashing_felts_methods::HASHING_FELTS_GUEST_ID;

fn main() {
    // let inputs = vec![Felt::from_hex_unchecked("0x6322CF2B00000000000000000000000000000000"); 5760];
    let inputs = get_5760_avg_base_fees_felt();

    // convert to f64 to print
    // let inputs_f64 = inputs
    //     .iter()
    //     .map(|f| convert_felt_to_f64(*f))
    //     .collect::<Vec<f64>>();
    // println!("inputs_f64: {:?}", inputs_f64);

    let input = HashingFeltInput { inputs };

    let (receipt, res) = hash_felts(input);

    receipt.verify(HASHING_FELTS_GUEST_ID).unwrap();

    println!("hash: {:?}", res.hash);
}
