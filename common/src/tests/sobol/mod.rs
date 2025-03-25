#[cfg(test)]
mod tests {

    use crate::convert_felt_to_f64;
    use crate::floating_point::add_twap_7d;
    use crate::floating_point::sobol;
    use crate::original::calculate_reserve_price;
    use crate::tests::mock::convert_data_to_vec_of_tuples;
    use crate::tests::mock::get_5760_avg_base_fees_felt;
    use rand::thread_rng;
    use rand_distr::Distribution;
    use statrs::distribution::ContinuousCDF;
    use statrs::distribution::Normal;

    fn standard_deviation(values: &[f64]) -> f64 {
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
        variance.sqrt()
    }

    #[test]
    fn test_sobol() {
        // initialization of data
        let n_periods = 720;
        let inputs_felt = get_5760_avg_base_fees_felt();

        // max return
        // let data_8_months = get_max_return_input_data();
        let data_8_months = inputs_felt
            .iter()
            .map(|x| convert_felt_to_f64(*x))
            .collect::<Vec<_>>();

        let data = data_8_months[data_8_months.len().saturating_sub(2160)..].to_vec();

        let twap_7d = add_twap_7d(&data).unwrap();
        let log_twap_7d: Vec<f64> = twap_7d.iter().map(|x| x.ln()).collect();
        let returns: Vec<f64> = log_twap_7d
            .windows(2)
            .map(|window| window[1] - window[0])
            .collect();

        let sigma = standard_deviation(&returns) * f64::sqrt(24.0 * 7.0);
        let dt = 1.0 / 24.0;

        // sobol
        let normal = Normal::new(0.0, sigma * f64::sqrt(dt)).unwrap();
        let mut sequence = sobol();

        let probs = sequence.by_ref().take(n_periods);
        let random_shocks_sobol: Vec<f64> = probs
            .map(|mut prob| {
                let pro = 1.0 - prob.pop().unwrap(); // ensure there are no zero values
                let x = normal.inverse_cdf(pro);
                println!("pro: {:?}", pro);
                // println!("x: {:?}", x);
                x
            })
            .collect();

        // original (ie. without sobol)
        let normal = Normal::new(0.0, sigma * f64::sqrt(dt)).unwrap();
        let mut rng = thread_rng();
        let random_shocks: Vec<f64> = (0..n_periods).map(|_| normal.sample(&mut rng)).collect();

        // println!("random_shocks_sobol: {:?}\n", random_shocks_sobol);
        // println!("random_shocks: {:?}", random_shocks);

        println!("random_shocks_sobol length: {:?}", random_shocks_sobol.len());
        println!("random_shocks length: {:?}", random_shocks.len());
    }
}
