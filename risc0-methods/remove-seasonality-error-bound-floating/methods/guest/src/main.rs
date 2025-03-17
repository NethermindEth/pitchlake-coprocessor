use common::floating_point::{calculate_remove_seasonality, error_bound_dvec, error_bound_f64};
use remove_seasonality_error_bound_floating_core::RemoveSeasonalityErrorBoundFloatingInput;
use risc0_zkvm::guest::env;

fn main() {
    let data: RemoveSeasonalityErrorBoundFloatingInput = env::read();
    let (slope, intercept, de_seasonalised_detrended_log_base_fee, season_param) =
        calculate_remove_seasonality(&data.data).unwrap();

    let is_within_tolerance_de_seasonalised_detrended_log_base_fee = error_bound_dvec(
        &data.de_seasonalised_detrended_log_base_fee,
        &de_seasonalised_detrended_log_base_fee,
        data.tolerance,
    );
    assert!(is_within_tolerance_de_seasonalised_detrended_log_base_fee);

    let is_within_tolerance_season_param =
        error_bound_dvec(&data.season_param, &season_param, data.tolerance);
    assert!(is_within_tolerance_season_param);

    let is_within_tolerance_slope = error_bound_f64(data.slope, slope, data.tolerance);
    assert!(is_within_tolerance_slope);

    let is_within_tolerance_intercept = error_bound_f64(data.intercept, intercept, data.tolerance);
    assert!(is_within_tolerance_intercept);

    env::commit(&data);
}
