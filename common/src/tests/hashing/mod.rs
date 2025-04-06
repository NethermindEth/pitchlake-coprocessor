#[cfg(test)]
mod tests {
    use starknet_core::types::Felt;

    use crate::{
        convert_felt_to_f64, generate_batched_hash_for_all_avg_base_fees,
        hash_of_hash_of_avg_base_fees, hashing::hash_avg_base_fees_in_batch,
    };

    #[test]
    fn test_hash_avg_base_fees_in_batch() {
        let input =
            vec![Felt::from_hex_unchecked("0x6322CF2B00000000000000000000000000000000"); 180];
        let result = hash_avg_base_fees_in_batch(&input);

        // convert u8 array to u32 array
        let mut u32_result = [0u32; 8];
        for i in 0..8 {
            u32_result[i] = u32::from_be_bytes(result[i * 4..(i + 1) * 4].try_into().unwrap());
        }

        // result gotten from cairo
        assert_eq!(
            u32_result,
            [
                1899064690, 2582884512, 4135155955, 798422494, 171281208, 1706336902, 1452034954,
                2201301487
            ]
        );
    }

    #[test]
    fn test_hash_of_hash_of_avg_base_fees() {
        let input =
            vec![Felt::from_hex_unchecked("0x6322CF2B00000000000000000000000000000000"); 180];
        let hash_of_avg_base_fees = hash_avg_base_fees_in_batch(&input);

        let input = vec![hash_of_avg_base_fees; 32];

        let hash_res = hash_of_hash_of_avg_base_fees(&input);
        let mut u32_result = [0u32; 8];
        for i in 0..8 {
            u32_result[i] = u32::from_be_bytes(hash_res[i * 4..(i + 1) * 4].try_into().unwrap());
        }

        // result gotten from cairo
        assert_eq!(
            u32_result,
            [
                2742400919, 928893470, 783653877, 3849600438, 875778534, 2708382470, 3020714680,
                2794336563
            ]
        );
        println!("{:?}", hash_res);
    }

    #[test]
    fn test_convert_felt_to_f64() {
        let input = Felt::from_hex_unchecked("0x1AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        let result = convert_felt_to_f64(input);
        assert_eq!((result - 1.6666666).abs() < 0.0000001, true);
    }

    #[test]
    fn test_generate_batched_hash_for_all_avg_base_fees() {
        let inputs =
            vec![Felt::from_hex_unchecked("0x6322CF2B00000000000000000000000000000000"); 5760];
        let hash_res = generate_batched_hash_for_all_avg_base_fees(&inputs);
        let mut u32_result = [0u32; 8];
        for i in 0..8 {
            u32_result[i] = u32::from_be_bytes(hash_res[i * 4..(i + 1) * 4].try_into().unwrap());
        }

        // result gotten from cairo
        assert_eq!(
            u32_result,
            [
                2742400919, 928893470, 783653877, 3849600438, 875778534, 2708382470, 3020714680,
                2794336563
            ]
        );
    }
}
