use eyre::{anyhow as err, Result};

pub fn add_twap_30d(data: &Vec<f64>) -> Result<Vec<f64>> {
    // DEVELOPER NOTE: Window size configuration for rolling TWAP
    // ===========================================================
    // Production uses 30-day window: 24 * 30 = 720 hours
    // POC uses 10-day window: 24 * 10 = 240 hours (to fit within 1440 hours total)
    //
    // With 1440 total hours in POC:
    // - 10-day TWAP produces: 1440 - 240 = 1200 values
    // - calculate_30d_returns needs these 1200 values > 240 to calculate returns
    // - Final output: 1200 - 240 = 960 return values for max calculation
    let required_window_size = if data.len() <= 2000 {
        24 * 10 // POC: 10-day window (240 hours)
    } else {
        24 * 30 // Production: 30-day window (720 hours)
    };

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
    // DEVELOPER NOTE: Return calculation period
    // ==========================================
    // Production: 30-day period (720 hours) for return calculations
    // POC: 10-day period (240 hours) to match the reduced TWAP window
    //
    // The period must match the window size used in add_twap_30d to ensure
    // we're comparing values that are the same interval apart.
    let period = if twap_30d.len() <= 1500 {
        24 * 10 // POC: 10-day period (240 hours)
    } else {
        24 * 30 // Production: 30-day period (720 hours)
    };

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

// DEVELOPER NOTE: Data Length Configuration
// ==========================================
// Production expects 8 months of data: 24 * 30 * 8 = 5760 hours
// POC expects 2 months of data: 24 * 30 * 2 = 1440 hours
//
// This assertion must align with:
// - methods/hashing-felts-methods/guest/src/main.rs (input length)
// - methods/core/src/lib.rs (ProofCompositionInput data volume)
pub fn calculate_max_returns(data: &Vec<f64>) -> f64 {
    // POC configuration: 2 months of data
    assert!(
        data.len() == 24 * 30 * 2,
        "Expected 1440 hours (2 months) for POC. Production requires 5760 hours (8 months)."
    );

    let twap_30d = add_twap_30d(data).unwrap();
    let returns = calculate_30d_returns(&twap_30d).unwrap();

    let max_return = returns
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(&0.0);

    *max_return
}
