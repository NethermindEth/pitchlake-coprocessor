use eyre::{anyhow as err, Result};
use statrs::statistics::Statistics;

pub fn add_twap_30d(data: &Vec<f64>) -> Result<Vec<f64>> {
    let required_window_size = 24 * 30;

    let n = data.len();

    if n < required_window_size {
        return Err(err!(
            "Insufficient data: At least {} data points are required, but only {} provided.",
            required_window_size,
            n
        ));
    }

    let mut twap_values = vec![];
    for i in required_window_size..n {
        let window_mean =
            data[i - required_window_size..i].iter().sum::<f64>() / required_window_size as f64;
        twap_values.push(window_mean);
    }

    Ok(twap_values)
}

pub fn calculate_30d_returns(twap_30d: &Vec<f64>) -> Result<Vec<f64>> {
    // 24 hours * 30 days = 720 hours
    let period = 24 * 30;

    if twap_30d.len() <= period {
        return Err(err!("Input vector must be longer than {} elements", period));
    }

    let mut returns = vec![];

    // Calculate returns for remaining values
    for i in period..twap_30d.len() {
        let current = twap_30d[i];
        let previous = twap_30d[i - period];
        let return_value = (current / previous) - 1.0;
        returns.push(return_value);
    }

    Ok(returns)
}

// expects 210 days (5,040 hours) of data
pub fn calculate_max_returns(data: &Vec<f64>) -> f64 {
    assert!(data.len() == 24 * 30 * 8);

    let twap_30d = add_twap_30d(data).unwrap();
    let returns = calculate_30d_returns(&twap_30d).unwrap();

    let std_dev = returns.std_dev();

    std_dev
}
