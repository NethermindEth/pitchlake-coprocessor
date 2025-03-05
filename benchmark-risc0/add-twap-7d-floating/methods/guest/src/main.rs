use benchmark::floating_point::add_twap_7d;
use nalgebra::DVector;
use risc0_zkvm::guest::env;

fn main() {
    let data: Vec<f64> = env::read();
    let res = add_twap_7d(&data).unwrap();

    // (Vec<f64>, Vec<f64>)
    env::commit(&(data, res));
}
