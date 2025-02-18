#[cfg(feature = "original")]
#[cfg(test)]
mod tests {
    use crate::{
        floating_point::add_twap_7d,
        original::{calculate_reserve_price, convert_input_to_df},
        tests::mock::get_first_period_data,
    };

    #[test]
    fn test_convert_input_to_df() {
        let data = get_first_period_data();
        let df = convert_input_to_df(&data);

        assert_eq!(df.height(), data.len());
        // let timestamps = df["timestamp"].i64().unwrap().to_vec();
        let base_fees_df = df
            .column("base_fee")
            .unwrap()
            .f64()
            .unwrap()
            .iter()
            .map(|x| x.unwrap())
            .collect::<Vec<f64>>();

        assert_eq!(base_fees_df, data.iter().map(|x| x.1).collect::<Vec<f64>>());

        let timestamps_df = df
            .column("date")
            .unwrap()
            .datetime()
            .unwrap()
            .to_vec()
            .iter()
            .map(|x| x.unwrap() / 1000)
            .collect::<Vec<_>>();
        assert_eq!(
            timestamps_df,
            data.iter().map(|x| x.0).collect::<Vec<i64>>()
        );

        let twap_7d = add_twap_7d(&data).unwrap();
        println!("twap_7d: {:?}", twap_7d[0]);

        let twap_7d_df = df
            .column("TWAP_7d")
            .unwrap()
            .f64()
            .unwrap()
            .iter()
            .map(|x| x.unwrap())
            .collect::<Vec<f64>>();
        println!("twap_7d_df: {:?}", twap_7d_df[0]);

        for (df, non_df) in twap_7d_df.iter().zip(twap_7d.iter()) {
            assert_eq!(df.ceil(), non_df.ceil());
        }
    }

    #[test]
    fn test_calculate_reserve_price() {
        let data = get_first_period_data();

        let res = calculate_reserve_price(&data);

        println!("res: {:?}", res);
    }
}
