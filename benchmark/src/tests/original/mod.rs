#[cfg(feature = "original")]
#[cfg(test)]
mod tests {
    use ndarray::s;

    use crate::{
        floating_point::{self, add_twap_7d},
        original::{calculate_reserve_price, convert_array1_to_dvec, convert_input_to_df},
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

    #[test]
    fn test_compare_simulate_price_results() {
        let data = get_first_period_data();
        let res = calculate_reserve_price(&data);

        println!("res.simulated_price");
        println!(
            "res.simulated_price[0]: {:?}",
            res.de_seasonalized_detrended_simulated_prices
                .row(0)
                .slice(s![..5])
        );
        println!(
            "res.simulated_price[1]: {:?}",
            res.de_seasonalized_detrended_simulated_prices
                .row(1)
                .slice(s![..5])
        );
        println!("res.reserve_price: {:?}", res.reserve_price);

        let log_bas_fee = convert_array1_to_dvec(res.de_seasonalised_detrended_log_base_fee);
        let pt = convert_array1_to_dvec(res.pt);
        let pt_1 = convert_array1_to_dvec(res.pt_1);
        let num_paths = 15000;
        let n_periods = 720;
        let (is_saddle_point, simulated_price) = floating_point::simulate_price_verify_position(
            &res.positions,
            &pt,
            &pt_1,
            5e-3,
            &log_bas_fee,
            n_periods,
            num_paths,
        );
        assert!(is_saddle_point);

        // println!("simulated_price: {:?}", simulated_price);
        println!("simulated_price");
        println!(
            "simulated_price[0]: {:?}",
            simulated_price
                .row(0)
                .columns(0, 5)
                .iter()
                .collect::<Vec<_>>()
        );
        println!(
            "simulated_price[1]: {:?}",
            simulated_price
                .row(1)
                .columns(0, 5)
                .iter()
                .collect::<Vec<_>>()
        );

        let reserve_price = floating_point::calculate_reserve_price(
            data[0].0,
            data[data.len() - 1].0,
            &convert_array1_to_dvec(res.season_param),
            &simulated_price,
            &res.twap_7d,
            res.slope,
            res.intercept,
            data.len(),
            num_paths,
            n_periods,
        )
        .unwrap();

        println!("reserve_price: {:?}", reserve_price);
    }
}
