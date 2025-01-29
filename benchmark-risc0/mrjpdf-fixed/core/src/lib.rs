use nalgebra::DVector;
use serde::{Deserialize, Serialize};
use simba::scalar::FixedI48F16;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MrjPdfFixedInput {
    params: Vec<FixedI48F16>,
    pt: DVector<FixedI48F16>,
    pt_1: DVector<FixedI48F16>,
}
