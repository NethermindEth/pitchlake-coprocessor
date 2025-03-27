#[cfg(test)]
mod tests {
    use ndarray::{stack, Axis};

    use crate::{
        common::dataframe::{
            convert_to_timestamp_base_fee_int_tuple, read_data_from_file,
            replace_timestamp_with_date, split_dataframe_into_periods,
        },
        convert_felt_to_f64,
        floating_point::{
            self, add_twap_7d, calculate_remove_seasonality, calculate_twap as calculate_twap_floating, calculated_reserve_price_from_simulated_log_prices, error_bound_dvec, error_bound_f64, error_bound_u64, error_bound_vec, pre_minimize
        },
        original::{
            calculate_reserve_price, calculate_twap::calculate_twap, convert_array1_to_dvec,
            convert_array2_to_dmatrix, convert_input_to_df,
        },
        tests::mock::{
            convert_data_to_vec_of_tuples, get_5760_avg_base_fees_felt, get_first_period_data,
            get_max_return_input_data,
        },
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

        let twap_7d = add_twap_7d(&data.iter().map(|x| x.1).collect()).unwrap();
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

        let res = calculate_reserve_price(&data, 15000, 720);

        println!("res: {:?}", res);
    }

    #[test]
    fn test_compare_simulate_price_results() {
        // let data = get_first_period_data();

        let inputs_felt = get_5760_avg_base_fees_felt();

        // max return
        // let data_8_months = get_max_return_input_data();
        let data_8_months = inputs_felt
            .iter()
            .map(|x| convert_felt_to_f64(*x))
            .collect::<Vec<_>>();

        let data = data_8_months[data_8_months.len().saturating_sub(2160)..].to_vec();

        let start_timestamp = 1708833600;  
        // let end_timestamp = 1708833600 + (3600 * 24 * 30 * 3); // as long as start to end timestamp is 90 days
        let data = convert_data_to_vec_of_tuples(data.clone(), start_timestamp);

        let res = calculate_reserve_price(&data, 15000, 720);
        println!("res.reserve_price: {:?}", res.reserve_price);
        // res.reserve_price: 1755519897.514507

        let de_seasonalised_detrended_log_base_fee =
            convert_array1_to_dvec(res.de_seasonalised_detrended_log_base_fee);
        let pt = convert_array1_to_dvec(res.pt);
        let pt_1 = convert_array1_to_dvec(res.pt_1);
        let num_paths = 4000;
        let n_periods = 720;
        let (is_saddle_point, simulated_price) = floating_point::simulate_price_verify_position(
            &res.positions,
            &pt,
            &pt_1,
            5e-2,
            &de_seasonalised_detrended_log_base_fee,
            n_periods,
            num_paths,
        );
        assert!(is_saddle_point);

        // let is_within_error_bound = floating_point::error_bound_matrix(
        //     &convert_array2_to_dmatrix(res.de_seasonalized_detrended_simulated_prices),
        //     &simulated_price,
        //     5.0,
        // );
        // assert!(is_within_error_bound);

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

        let is_within_tolerance_reserve_price =
            error_bound_u64(reserve_price, res.reserve_price, 5.0);
        assert!(is_within_tolerance_reserve_price);
        // reserve_price: 1765847736.6691935 (num_paths: 15,000)
        // reserve_price: 1710956542.6769266 (num_paths: 4,000)

        let original_reserve_price = calculated_reserve_price_from_simulated_log_prices(
            &convert_array2_to_dmatrix(res.simulated_log_prices),
            &res.twap_7d,
            n_periods,
        )
        .unwrap();

        println!("original_reserve_price: {:?}", original_reserve_price);
        // original_reserve_price: 1735924412.5244353

    }

    #[test]
    fn test_convert_array2_to_dmatrix() {
        let first_column = vec![1, 2, 3];
        let second_column = vec![4, 5, 6];
        let third_column = vec![7, 8, 9];

        let array2 = stack![Axis(1), first_column, second_column, third_column];
        let dmatrix = convert_array2_to_dmatrix(array2);

        assert_eq!(
            dmatrix.row(0).iter().copied().collect::<Vec<_>>(),
            vec![1, 4, 7]
        );
        assert_eq!(
            dmatrix.row(1).iter().copied().collect::<Vec<_>>(),
            vec![2, 5, 8]
        );
        assert_eq!(
            dmatrix.row(2).iter().copied().collect::<Vec<_>>(),
            vec![3, 6, 9]
        );
    }

    #[test]
    fn test_compare_calculated_reserve_price_from_simulated_log_prices() {
        let num_paths = 4000;
        let n_periods = 720;
        let data = get_first_period_data();
        let res = calculate_reserve_price(&data, num_paths, n_periods);
        println!("original reserve_price: {:?}", res.reserve_price);

        let reserve_price = floating_point::calculated_reserve_price_from_simulated_log_prices(
            &convert_array2_to_dmatrix(res.simulated_log_prices),
            &res.twap_7d,
            n_periods,
        )
        .unwrap();

        println!("reserve_price: {:?}", reserve_price);
    }

    #[test]
    fn test_compare_add_twap_7d() {
        let data = get_first_period_data();
        let res = calculate_reserve_price(&data, 15000, 720);
        let twap_7d = add_twap_7d(&data.iter().map(|x| x.1).collect()).unwrap();

        let percentage_tolerance = 0.00001;
        let is_within_tolerance = error_bound_vec(&twap_7d, &res.twap_7d, percentage_tolerance);
        assert!(is_within_tolerance);
    }

    #[test]
    fn test_compare_remove_seasonality() {
        let data = get_first_period_data();
        let res = calculate_reserve_price(&data, 15000, 720);

        let (slope, intercept, de_seasonalised_detrended_log_base_fee, season_param) =
            calculate_remove_seasonality(&data.iter().map(|x| x.1).collect()).unwrap();

        let is_within_tolerance_de_seasonalised_detrended_log_base_fee = error_bound_dvec(
            &convert_array1_to_dvec(res.de_seasonalised_detrended_log_base_fee),
            &de_seasonalised_detrended_log_base_fee,
            0.00001,
        );
        assert!(is_within_tolerance_de_seasonalised_detrended_log_base_fee);

        let is_within_tolerance_season_param = error_bound_dvec(
            &convert_array1_to_dvec(res.season_param),
            &season_param,
            0.00001,
        );
        assert!(is_within_tolerance_season_param);

        let is_within_tolerance_slope = error_bound_f64(res.slope, slope, 0.00001);
        assert!(is_within_tolerance_slope);

        let is_within_tolerance_intercept = error_bound_f64(res.intercept, intercept, 0.00001);
        assert!(is_within_tolerance_intercept);
    }

    #[test]
    fn test_compare_calculate_pt_pt1() {
        let data = get_first_period_data();
        let res = calculate_reserve_price(&data, 15000, 720);

        let (pt, pt_1, _var_pt) = pre_minimize(&convert_array1_to_dvec(
            res.de_seasonalised_detrended_log_base_fee,
        ));

        let is_within_tolerance_pt =
            error_bound_dvec(&pt, &convert_array1_to_dvec(res.pt), 0.00001);
        assert!(is_within_tolerance_pt);

        let is_within_tolerance_pt_1 =
            error_bound_dvec(&pt_1, &convert_array1_to_dvec(res.pt_1), 0.00001);
        assert!(is_within_tolerance_pt_1);
    }

    #[test]
    fn test_compare_calculate_twap() {
        let df = read_data_from_file("data.csv");
        let df = replace_timestamp_with_date(df).unwrap();
        let first_period = split_dataframe_into_periods(df, 3)
            .unwrap()
            .into_iter()
            .take(1)
            .next()
            .unwrap();
        // calculate using original algorithm using base_fee for each block
        let data: Vec<(i64, i64)> = convert_to_timestamp_base_fee_int_tuple(first_period);
        let twap_original = calculate_twap(&data);

        println!("twap: {:?}", twap_original);

        // calculated using zkvm algorithm using average hourly base fee
        let data = get_first_period_data();
        let twap = calculate_twap_floating(&data.iter().map(|x| x.1).collect::<Vec<f64>>());
        println!("twap: {:?}", twap);

        // in this case, the tolerance is 0.5 because the data is not too close when using avg as an input
        // vs. using base_fee for each block
        // but it is close enough
        let is_within_tolerance = error_bound_f64(twap, twap_original, 0.5);
        assert!(is_within_tolerance);
    }

    #[test]
    fn test_compare_calculate_max_return() {
        // tested manually against python's result
        let data = get_max_return_input_data();
        let max_return =
            floating_point::calculate_max_returns(&data.iter().map(|x| x.1).collect::<Vec<f64>>());
        // max_return: 1.544626972559826
        println!("max_return: {:?}", max_return);
        assert!((max_return - 1.544).abs() < 0.001);
    }

    #[test]
    fn test_compare_add_twap_30d() {
        // tested manually against python's result
        let data = get_max_return_input_data();
        let twap_30d =
            floating_point::add_twap_30d(&data.iter().map(|x| x.1).collect::<Vec<f64>>()).unwrap();
        println!("twap_30d: {:?}", twap_30d);
    }
}
