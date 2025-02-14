#[cfg(test)]
mod tests {
    use crate::{original::convert_input_to_df, tests::mock::get_first_period_data};

    #[test]
    fn test_convert_input_to_df() {
        let data = get_first_period_data();
        let df = convert_input_to_df(&data);
        println!("df: {:?}", df);
    }
}
