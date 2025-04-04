pub mod floating_point;
#[cfg(feature = "original")]
pub mod original;

pub mod solution;
pub use solution::*;
pub mod hashing;
pub use hashing::*;

pub mod common;
pub mod tests;

use eyre::Result;

pub fn hex_string_to_f64(hex_str: &String) -> Result<f64> {
    let stripped = hex_str.trim_start_matches("0x");
    u128::from_str_radix(stripped, 16)
        .map(|value| value as f64)
        .map_err(|e| eyre::eyre!("Error converting hex string '{}' to f64: {}", hex_str, e))
}

pub fn is_saddle_point(gradient: &[f64], tolerance: f64) -> bool {
    gradient.iter().all(|dx| dx.abs() <= tolerance)
}
