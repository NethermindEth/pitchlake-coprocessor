use eyre::{anyhow as err, Result};
use nalgebra::{DMatrix, DVector};

use super::season_matrix;

fn fit_linear_regression(x: &[f64], y: &[f64]) -> Result<(f64, f64)> {
    if x.len() != y.len() {
        return Err(eyre::eyre!("Input arrays x and y must have the same length.").into());
    }

    let n = x.len();
    let x_vec = DVector::from_row_slice(x);
    let y_vec = DVector::from_row_slice(y);

    let mut design_matrix = DMatrix::zeros(n, 2);
    design_matrix.set_column(0, &DVector::from_element(n, 1.0));
    design_matrix.set_column(1, &x_vec);

    let solution = (&design_matrix.transpose() * &design_matrix)
        .try_inverse()
        .ok_or_else(|| eyre::eyre!("Singular matrix"))?
        * &design_matrix.transpose()
        * y_vec;

    Ok((solution[1], solution[0]))
}

fn predict(x: &[f64], slope: f64, intercept: f64) -> DVector<f64> {
    DVector::from_iterator(x.len(), x.iter().map(|&xi| slope * xi + intercept))
}

fn discover_trend(log_base_fee: &[f64]) -> Result<(f64, f64, Vec<f64>)> {
    let time_index: Vec<f64> = (0..log_base_fee.len()).map(|i| i as f64).collect();
    let (slope, intercept) = fit_linear_regression(&time_index, log_base_fee)?;
    let trend_values = predict(&time_index, slope, intercept);

    Ok((slope, intercept, trend_values.as_slice().to_vec()))
}

fn compute_log_of_base_fees(base_fees: &Vec<&f64>) -> Result<Vec<f64>> {
    Ok(base_fees.iter().map(|&x| x.ln()).collect())
}

fn remove_seasonality(
    detrended_log_base_fee: &DVector<f64>,
    data: &[(i64, f64)],
) -> Result<(DVector<f64>, DVector<f64>)> {
    let start_timestamp = data
        .first()
        .ok_or_else(|| err!("Missing start timestamp"))?
        .0;

    // hack:
    let t_series = DVector::from_iterator(
        data.len(),
        data.iter()
            .map(|(timestamp, _)| (*timestamp - start_timestamp) as f64 / 3600.0),
    );

    let c = season_matrix(t_series.clone());

    let epsilon = 1e-300;
    let season_param = lstsq::lstsq(&c, &detrended_log_base_fee, epsilon)
        .unwrap()
        .solution;
    let season = &c * &season_param;

    let de_seasonalised_detrended_log_base_fee = detrended_log_base_fee - season;
    Ok((de_seasonalised_detrended_log_base_fee, season_param))
}

// assume data is sorted by timestamp
pub fn calculate_remove_seasonality(
    data: &[(i64, f64)],
) -> Result<(f64, f64, DVector<f64>, DVector<f64>)> {
    let fees: Vec<&f64> = data.iter().map(|x| &x.1).collect();

    let log_base_fee = compute_log_of_base_fees(&fees)?;
    let (slope, intercept, trend_values) = discover_trend(&log_base_fee)?;

    let detrended_log_base_fee: DVector<f64> = DVector::from_iterator(
        log_base_fee.len(),
        log_base_fee
            .iter()
            .zip(&trend_values)
            .map(|(log_base_fee, trend)| log_base_fee - trend),
    );

    let (de_seasonalised_detrended_log_base_fee, season_param) =
        remove_seasonality(&detrended_log_base_fee, &data)?;

    Ok((
        slope,
        intercept,
        de_seasonalised_detrended_log_base_fee,
        season_param,
    ))
}
