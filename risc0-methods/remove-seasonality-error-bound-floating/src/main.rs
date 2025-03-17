use common::{
    original::{self, convert_array1_to_dvec},
    tests::mock::get_first_period_data,
};
use remove_seasonality_error_bound_floating::remove_seasonality_error_bound;
use remove_seasonality_error_bound_floating_core::RemoveSeasonalityErrorBoundFloatingInput;
use remove_seasonality_error_bound_floating_methods::REMOVE_SEASONALITY_ERROR_BOUND_FLOATING_GUEST_ID;

fn main() {
    let data = get_first_period_data();
    // run rust code in host
    // ensure convergence in host
    let res = original::calculate_reserve_price(&data, 15000, 720);

    let input = RemoveSeasonalityErrorBoundFloatingInput {
        data: data.iter().map(|x| x.1).collect(),
        slope: res.slope,
        intercept: res.intercept,
        de_seasonalised_detrended_log_base_fee: convert_array1_to_dvec(
            res.de_seasonalised_detrended_log_base_fee,
        ),
        season_param: convert_array1_to_dvec(res.season_param),
        tolerance: 0.00001, // 0.00001%
    };

    let (receipt, _res) = remove_seasonality_error_bound(input);

    receipt
        .verify(REMOVE_SEASONALITY_ERROR_BOUND_FLOATING_GUEST_ID)
        .unwrap();
}
