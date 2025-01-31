use risc0_zkvm::guest::env;
use benchmark::floating_point::neg_log_likelihood;
use neg_log_floating_core::NegLogFloatingInput;

fn main() {
    let query: NegLogFloatingInput = env::read();
    let res = neg_log_likelihood(&query.params, &query.pt, &query.pt_1);
    env::commit(&res);
}
