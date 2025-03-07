use hashing_felts::hash_felts;
use hashing_felts_core::HashingFeltInput;
use hashing_felts_methods::HASHING_FELTS_GUEST_ID;
use starknet_core::types::Felt;

fn main() {
    let inputs = vec![Felt::from_hex_unchecked("0x6322CF2B00000000000000000000000000000000"); 5760];

    let input = HashingFeltInput { inputs };

    let (receipt, res) = hash_felts(input);

    receipt.verify(HASHING_FELTS_GUEST_ID).unwrap();

    println!("hash: {:?}", res.hash);
}
