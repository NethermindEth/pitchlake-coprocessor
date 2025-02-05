use benchmark::fixed_point::FixedPoint;
use nalgebra::DVector;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MrjPdfFixedInput {
    pub params: Vec<FixedPoint>,
    pub pt: DVector<FixedPoint>,
    pub pt_1: DVector<FixedPoint>,
}
