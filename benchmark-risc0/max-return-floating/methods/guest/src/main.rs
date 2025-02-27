use benchmark::floating_point::calculate_max_returns;
use max_return_floating_core::MaxReturnInput;
use risc0_zkvm::guest::env;

fn main() {
    let input: MaxReturnInput = env::read();

    let max_return = calculate_max_returns(&input.data);
    env::commit(&(input, max_return));
}
