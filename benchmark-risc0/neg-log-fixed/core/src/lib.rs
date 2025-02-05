use nalgebra::DVector;
use serde::{Deserialize, Serialize};
use benchmark::fixed_point::FixedPoint;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NegLogFixedInput {
    pub params: Vec<FixedPoint>,
    pub pt: DVector<FixedPoint>,
    pub pt_1: DVector<FixedPoint>,
}

