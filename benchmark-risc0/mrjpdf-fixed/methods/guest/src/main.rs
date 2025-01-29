use risc0_zkvm::guest::env;

fn main() {
    let query: u64 = env::read();
    env::commit(&query);
}
