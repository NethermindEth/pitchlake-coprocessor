use nalgebra::DVector;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NegLogFloatingInput {
    pub params: Vec<f64>,
    pub pt: DVector<f64>,
    pub pt_1: DVector<f64>,
}
