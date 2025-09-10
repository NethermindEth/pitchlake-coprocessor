use core::{ProofCompositionInput, ProofCompositionOutput};
use garaga_rs::{
    calldata::full_proof_with_hints::groth16::{
        get_groth16_calldata_felt, risc0_utils::get_risc0_vk, Groth16Proof,
    },
    definitions::CurveID,
};
use mock_proof_composition_methods::MOCK_PROOF_COMPOSITION_GUEST_ELF;
use nalgebra::DVector;
use risc0_ethereum_contracts::encode_seal;
use risc0_zkvm::{compute_image_id, default_prover, ExecutorEnv, ProverOpts, VerifierContext};

fn main() {
    dotenv::dotenv().ok();

    let data = ProofCompositionInput {
        data_8_months: vec![0.1, 0.2, 0.3, 0.4, 0.5],
        data_8_months_hash: [
            0x12345678, 0x23456789, 0x3456789a, 0x456789ab, 0x56789abc, 0x6789abcd, 0x789abcde,
            0x89abcdef,
        ],
        data_8_months_start_timestamp: 1651363200, // 2022-05-01 (8 months earlier)
        data_8_months_end_timestamp: 1704067200,   // 2024-01-01
        start_timestamp: 1672531200,               // 2023-01-01 (3 months)
        end_timestamp: 1704067200,                 // 2024-01-01
        positions: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        pt: DVector::from_vec(vec![0.1, 0.2, 0.3]),
        pt_1: DVector::from_vec(vec![0.2, 0.3, 0.4]),
        gradient_tolerance: 0.001,
        de_seasonalised_detrended_log_base_fee: DVector::from_vec(vec![0.5, 0.6, 0.7]),
        n_periods: 24,
        num_paths: 100,
        season_param: DVector::from_vec(vec![0.8, 0.9, 1.0]),
        twap_7d: vec![1.1, 1.2, 1.3],
        slope: 0.05,
        intercept: 1.5,
        reserve_price: 2.5,
        floating_point_tolerance: 0.0001,
        reserve_price_tolerance: 0.01,
        twap_tolerance: 0.05,
        twap_result: 1.25,
        max_return: 0.3,
    };

    let env = ExecutorEnv::builder()
        .write(&data)
        .unwrap()
        .build()
        .unwrap();

    let receipt = default_prover()
        .prove_with_ctx(
            env,
            &VerifierContext::default(),
            MOCK_PROOF_COMPOSITION_GUEST_ELF,
            &ProverOpts::groth16(),
        )
        .unwrap()
        .receipt;

    let encoded_seal = encode_seal(&receipt).unwrap();

    let image_id = compute_image_id(MOCK_PROOF_COMPOSITION_GUEST_ELF).unwrap();

    let journal = receipt.journal.bytes.clone();
    println!("JOURNAL: {:?}", journal);

    let decoded_journal = receipt.journal.decode::<ProofCompositionOutput>();
    println!("DECODED JOURNAL: {:?}", decoded_journal);

    let groth16_proof =
        Groth16Proof::from_risc0(encoded_seal, image_id.as_bytes().to_vec(), journal.clone());

    let calldata =
        get_groth16_calldata_felt(&groth16_proof, &get_risc0_vk(), CurveID::BN254).unwrap();
    println!("CALLLDATA: {:?}", calldata);
}
