use add_twap_7d_error_bound_floating::add_twap_7d_error_bound;
use add_twap_7d_error_bound_floating_methods::ADD_TWAP_7D_ERROR_BOUND_FLOATING_GUEST_ID;
use common::{original, tests::mock::get_first_period_data};
use core::AddTwap7dErrorBoundFloatingInput;

fn main() {
    // get only first period of (timestamp avg_gas_fee)
    let data = get_first_period_data();
    // run rust code in host
    // ensure convergence in host
    let res = original::calculate_reserve_price(&data, 15000, 720);

    let input = AddTwap7dErrorBoundFloatingInput {
        data: data.iter().map(|x| x.1).collect(),
        twap_7d: res.twap_7d,
        tolerance: 0.00001, // 0.00001%
    };

    let (receipt, _res) = add_twap_7d_error_bound(input);

    receipt
        .verify(ADD_TWAP_7D_ERROR_BOUND_FLOATING_GUEST_ID)
        .unwrap();
}
