use mrjpdf_fixed_core::MrjPdfFixedInput;
use benchmark::fixed_point::mrjpdf;
use risc0_zkvm::guest::env;

fn main() {
    let query: MrjPdfFixedInput = env::read();
    let res = mrjpdf(&query.params, &query.pt, &query.pt_1);
    env::commit(&res);
}
