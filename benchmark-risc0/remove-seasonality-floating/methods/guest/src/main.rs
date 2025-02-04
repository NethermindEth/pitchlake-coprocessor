use benchmark::floating_point::calculate_remove_seasonality;
use risc0_zkvm::guest::env;

fn main() {
    // let query: NegLogFloatingInput = env::read();
    let data: Vec<(i64, f64)> = env::read();
    let res = calculate_remove_seasonality(&data);

    // return (f64, f64, DVector<f64>, DVector<f64>)
    env::commit(&res.unwrap());
}