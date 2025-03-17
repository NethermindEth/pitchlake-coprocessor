use eyre::{anyhow as err, Result};
use nalgebra::DVector;

pub fn add_twap_7d(data: &Vec<f64>) -> Result<Vec<f64>> {
    let required_window_size = 24 * 7;
    let n = data.len();

    if n < required_window_size {
        return Err(err!(
            "Insufficient data: At least {} data points are required, but only {} provided.",
            required_window_size,
            n
        ));
    }

    let values = DVector::from_iterator(n, data.iter().map(|&x| x));
    let mut twap_values = Vec::with_capacity(n);

    for i in 0..n {
        let window_start = if i >= required_window_size {
            i - required_window_size + 1
        } else {
            0
        };
        let window_mean = values.rows(window_start, i - window_start + 1).mean();
        twap_values.push(window_mean);
    }

    Ok(twap_values)
}
