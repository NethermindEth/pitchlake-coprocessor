use serde::{Deserialize, Serialize};
use starknet_core::types::Felt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HashingFeltInput {
    pub inputs: Vec<Felt>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HashingFeltOutput {
    pub hash: Vec<u8>,
    pub f64_inputs: Vec<f64>,
}
