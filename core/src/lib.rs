use serde::{Deserialize, Serialize};
use simba::scalar::FixedI48F16;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VolatilityInputsFixedPointSimba {
    pub base_fee_per_gases: Vec<Option<FixedI48F16>>,
    pub ln_results: Vec<FixedI48F16>,
}
