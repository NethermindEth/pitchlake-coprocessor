use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MaxReturnInput {
    pub data: Vec<f64>,
}
