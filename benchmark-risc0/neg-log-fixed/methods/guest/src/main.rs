use neg_log_fixed_core::NegLogFixedInput;
use benchmark::fixed_point::neg_log_likelihood;
use risc0_zkvm::guest::env;

fn main() {
    let query: NegLogFixedInput = env::read();
    let res = neg_log_likelihood(&query.params, &query.pt, &query.pt_1);
    env::commit(&res);
}
