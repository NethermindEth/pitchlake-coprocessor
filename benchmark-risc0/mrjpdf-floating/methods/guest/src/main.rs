use mrjpdf_floating_core::MrjPdfFloatingInput;
use benchmark::floating_point::mrjpdf;
use risc0_zkvm::guest::env;

fn main() {
    let query: MrjPdfFloatingInput = env::read();
    let res = mrjpdf(&query.params, &query.pt, &query.pt_1);
    env::commit(&res);
}
