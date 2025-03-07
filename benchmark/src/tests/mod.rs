mod floating_point;
pub mod mock;
mod original;
mod hashing;

#[cfg(test)]
mod tests {
    use crate::fixed_point::{
        mrjpdf as mrjpdf_fixed, neg_log_likelihood as neg_log_likelihood_fixed, FixedPoint,
    };
    use crate::tests::mock::generate_inputs;
    use nalgebra::DVector;

    #[test]
    fn test_mrjpdf_fixed() {
        let (params, pt_data, pt1_data) = generate_inputs();
        let params = params
            .iter()
            .map(|x| FixedPoint::from_num(*x))
            .collect::<Vec<_>>();

        let pt_data = pt_data
            .iter()
            .map(|x| FixedPoint::from_num(*x))
            .collect::<Vec<_>>();

        let pt1_data = pt1_data
            .iter()
            .map(|x| FixedPoint::from_num(*x))
            .collect::<Vec<_>>();

        let pt = DVector::from_vec(pt_data);
        let pt1 = DVector::from_vec(pt1_data);

        let result = mrjpdf_fixed(&params, &pt, &pt1);
        println!("{:?}", result);
    }

    #[test]
    fn test_neg_log_likelihood_fixed() {
        let (params, pt_data, pt1_data) = generate_inputs();
        let params = params
            .iter()
            .map(|x| FixedPoint::from_num(*x))
            .collect::<Vec<_>>();

        let pt_data = pt_data
            .iter()
            .map(|x| FixedPoint::from_num(*x))
            .collect::<Vec<_>>();

        let pt1_data = pt1_data
            .iter()
            .map(|x| FixedPoint::from_num(*x))
            .collect::<Vec<_>>();

        let pt = DVector::from_vec(pt_data);
        let pt1 = DVector::from_vec(pt1_data);

        let result = neg_log_likelihood_fixed(&params, &pt, &pt1);
        println!("result: {}", result);
    }
}
