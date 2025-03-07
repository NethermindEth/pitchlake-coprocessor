use sha2::Digest;
use starknet_core::types::{Felt, U256};

pub fn hash_avg_base_fees_in_batch(input: &Vec<Felt>) -> Vec<u8> {
    let mut res_array = vec![];
    for i in input {
        let u32_array = convert_felt_to_u32_array(*i);
        res_array.append(&mut u32_array.to_vec());
    }

    let hash_input = convert_32_bytes_words_to_hash_input(&res_array);

    let hash = sha2::Sha256::digest(&hash_input);
    hash.to_vec()
}

fn convert_felt_to_u32_array(input: Felt) -> [u32; 8] {
    let input_le_bytes = input.to_bytes_le();
    let mut input_array = [0u32; 8];
    for i in 0..8 {
        input_array[i] = u32::from_le_bytes(input_le_bytes[i * 4..(i + 1) * 4].try_into().unwrap());
    }

    input_array
}

fn convert_32_bytes_words_to_hash_input(input: &Vec<u32>) -> Vec<u8> {
    let mut result = vec![];
    for word in input {
        result.append(&mut word.to_be_bytes().to_vec());
    }
    result
}

pub fn hash_of_hash_of_avg_base_fees(hashes: &Vec<Vec<u8>>) -> Vec<u8> {
    let mut input_array = vec![];
    for hash in hashes {
        input_array.append(&mut hash.to_vec());
    }

    let hash = sha2::Sha256::digest(&input_array);
    println!("{:x}", hash);
    hash.to_vec()
}

pub fn convert_felt_to_f64(input: Felt) -> f64 {
    let input_u256 = U256::from(input);

    const TWO_POW_128: f64 = 340282366920938463463374607431768211456.0;
    let decimal = input_u256.low() as f64 / TWO_POW_128;
    input_u256.high() as f64 + decimal
}
