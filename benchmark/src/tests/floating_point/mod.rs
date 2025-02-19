#[cfg(test)]
mod tests {
    use nalgebra::{DMatrix, DVector};

    use crate::{
        floating_point::{error_bound_matrix, mrjpdf, neg_log_likelihood},
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

        let result = error_bound_matrix(n1, n2, error_tolerance);
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

        let result = error_bound_matrix(n1, n2, error_tolerance);
        assert!(!result);
    }
}
