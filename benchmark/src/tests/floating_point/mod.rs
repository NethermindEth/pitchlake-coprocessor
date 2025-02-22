#[cfg(test)]
mod tests {
    use nalgebra::{DMatrix, DVector};

    use crate::{
        floating_point::{
            error_bound_matrix, error_bound_simulated_log_prices, error_bound_vec, mrjpdf, neg_log_likelihood
        },
        tests::mock::generate_inputs,
    };

    #[test]
    fn test_mrjpdf_floating() {
        let (param, pt_data, pt1_data) = generate_inputs();

        let pt = DVector::from_vec(pt_data);
        let pt1 = DVector::from_vec(pt1_data);

        let result = mrjpdf(&param, &pt, &pt1);
        println!("{:?}", result);
    }

    #[test]
    fn test_neg_log_likelihood_floating() {
        let (params, pt_data, pt1_data) = generate_inputs();

        let pt = DVector::from_vec(pt_data);

        let pt1 = DVector::from_vec(pt1_data);

        let result = neg_log_likelihood(&params, &pt, &pt1);
        println!("result: {}", result);
    }

    #[test]
    fn test_error_bound_matrix_within_tolerance() {
        let rows = 10;
        let cols = 20;
        let mut n1 = DMatrix::zeros(rows, cols);
        let mut n2 = DMatrix::zeros(rows, cols);

        let mut e = 0.25;
        for i in 0..rows {
            for j in 0..cols {
                n1[(i, j)] = e;
                n2[(i, j)] = e;

                e += 0.20;
            }
        }

        let error_tolerance = 5.0;

        let modified_i_index = 5;
        let modified_j_index = 5;

        // Modify one element to be 4.9% less than original
        let original_value = n2[(modified_i_index, modified_j_index)];
        n2[(modified_i_index, modified_j_index)] = original_value * 0.951; // 4.9% less

        let result = error_bound_matrix(&n1, &n2, error_tolerance);
        assert!(result);
    }

    #[test]
    fn test_error_bound_matrix_outside_tolerance() {
        let rows = 10;
        let cols = 20;
        let mut n1 = DMatrix::zeros(rows, cols);
        let mut n2 = DMatrix::zeros(rows, cols);

        let mut e = 0.25;
        for i in 0..rows {
            for j in 0..cols {
                n1[(i, j)] = e;
                n2[(i, j)] = e;

                e += 0.20;
            }
        }

        let error_tolerance = 5.0;

        let modified_i_index = 5;
        let modified_j_index = 5;

        // Modify one element to be 5.1% less than original
        let original_value = n2[(modified_i_index, modified_j_index)];
        n2[(modified_i_index, modified_j_index)] = original_value * 0.949; // 5.1% less

        let result = error_bound_matrix(&n1, &n2, error_tolerance);
        assert!(!result);
    }

    #[test]
    fn test_error_bound_simulated_log_prices_within_matrix_tolerance() {
        let rows = 10;
        let cols = 30;
        let mut n1 = DMatrix::zeros(rows, cols);
        let mut n2 = DMatrix::zeros(rows, cols);

        let mut e = 0.25;
        for i in 0..rows {
            for j in 0..cols {
                n1[(i, j)] = e;
                n2[(i, j)] = e;

                e += 0.20;
            }
        }

        let matrix_tolerance = 5.0; // 15 elements
        let element_wise_tolerance = 10.0;

        // modify 14 elements, less than matrix_tolerance
        n2[(0, 0)] = n2[(0, 0)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(0, 1)] = n2[(0, 1)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(0, 2)] = n2[(0, 2)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(0, 3)] = n2[(0, 3)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(0, 4)] = n2[(0, 4)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;

        n2[(0, 5)] = n2[(0, 5)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(1, 0)] = n2[(1, 0)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(1, 1)] = n2[(1, 1)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(1, 2)] = n2[(1, 2)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(1, 3)] = n2[(1, 3)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;

        n2[(1, 4)] = n2[(1, 4)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(1, 5)] = n2[(1, 5)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(2, 0)] = n2[(2, 0)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(2, 1)] = n2[(2, 1)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        // n2[(2, 2)] = n2[(2, 2)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;

        let result =
            error_bound_simulated_log_prices(&n1, &n2, element_wise_tolerance, matrix_tolerance);
        assert!(result);
    }

    #[test]
    fn test_error_bound_simulated_log_prices_outside_matrix_tolerance() {
        let rows = 10;
        let cols = 30;
        let mut n1 = DMatrix::zeros(rows, cols);
        let mut n2 = DMatrix::zeros(rows, cols);

        let mut e = 0.25;
        for i in 0..rows {
            for j in 0..cols {
                n1[(i, j)] = e;
                n2[(i, j)] = e;

                e += 0.20;
            }
        }

        let matrix_tolerance = 5.0; // 15 elements
        let element_wise_tolerance = 10.0;

        // modify 14 elements, less than matrix_tolerance
        n2[(0, 0)] = n2[(0, 0)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(0, 1)] = n2[(0, 1)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(0, 2)] = n2[(0, 2)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(0, 3)] = n2[(0, 3)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(0, 4)] = n2[(0, 4)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;

        n2[(0, 5)] = n2[(0, 5)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(1, 0)] = n2[(1, 0)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(1, 1)] = n2[(1, 1)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(1, 2)] = n2[(1, 2)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(1, 3)] = n2[(1, 3)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;

        n2[(1, 4)] = n2[(1, 4)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(1, 5)] = n2[(1, 5)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(2, 0)] = n2[(2, 0)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(2, 1)] = n2[(2, 1)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;
        n2[(2, 2)] = n2[(2, 2)] * (100.0 + element_wise_tolerance + 1.0) / 100.0;

        let result =
            error_bound_simulated_log_prices(&n1, &n2, element_wise_tolerance, matrix_tolerance);
        assert!(!result);
    }

    #[test]
    fn test_error_bound_vec_within_tolerance() {
        let target = vec![1.0, 2.0, 3.0];
        let calculated = vec![1.0001, 2.0, 3.0];
        let tolerance = 1.0;
        let result = error_bound_vec(&target, &calculated, tolerance);
        assert!(result);
    }

    #[test]
    fn test_error_bound_vec_outside_tolerance() {
        let target = vec![1.0, 2.0, 3.0];
        let calculated = vec![1.1, 2.0, 3.0];
        let tolerance = 1.0;
        let result = error_bound_vec(&target, &calculated, tolerance);
        assert!(!result);
    }
}
