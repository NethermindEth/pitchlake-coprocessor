use eyre::Result;
use nalgebra::{DVector, RealField};
use num_traits::Zero;
use simba::scalar::{ComplexField, FixedI48F16};

// Copy pasted from `left_scalar_mul_impl` in `nalgebra/src/base/ops.rs`
// if possible, we should modify the `nalgebra` crate to support scalar-vector multiplication
// however when i tried to do this, i couldnt get this current file to 'detect' the changes
// Notice that we have `rhs.clone()` which is inefficient. however even in the original code,
// the same cloning approach is used.
fn mul(lhs: FixedI48F16, rhs: &DVector<FixedI48F16>) -> DVector<FixedI48F16> {
    let mut res = rhs.clone().into_owned();
    for rhs in res.as_mut_slice().iter_mut() {
        *rhs *= lhs
    }
    res
}

pub fn powi(x: FixedI48F16, y: usize) -> FixedI48F16 {
    let mut result = FixedI48F16::from_num(1);
    for _ in 0..y {
        result = result * x;
    }
    result
}

pub fn natural_log(x: FixedI48F16) -> Result<FixedI48F16> {
    if x <= FixedI48F16::zero() {
        return Err(eyre::eyre!("Cannot take logarithm of non-positive number"));
    }
    let mut power = 0i32;
    let two = FixedI48F16::from_num(2);
    let one = FixedI48F16::from_num(1);
    let mut val = x;
    while val >= two {
        val = val / two;
        power += 1;
    }
    while val < one {
        val = val * two;
        power -= 1;
    }
    let base_ln = FixedI48F16::ln_2() * FixedI48F16::from_num(power);
    let frac = val - one;
    let frac_contribution = frac * FixedI48F16::ln_2();
    Ok(base_ln + frac_contribution)
}

pub fn mrjpdf(
    params: &[FixedI48F16],
    pt: &DVector<FixedI48F16>,
    pt_1: &DVector<FixedI48F16>,
) -> DVector<FixedI48F16> {
    let (a, phi, mu_j, sigma_sq, sigma_sq_j, lambda) = (
        params[0], params[1], params[2], params[3], params[4], params[5],
    );

    let phi_mult_pt_1 = mul(phi, pt_1);
    // NOTE: use of cloning here will result in inefficient code, however im not sure there is a way to do this without cloning
    let diff1 = pt
        - (DVector::from_element(pt.len(), a)
            + phi_mult_pt_1.clone()
            + DVector::from_element(pt.len(), mu_j));
    let diff2 = pt - (DVector::from_element(pt.len(), a) + phi_mult_pt_1);

    let term1_rhs = (-diff1.map(|x| powi(x, 2))
        / (FixedI48F16::from_num(2) * (sigma_sq + sigma_sq_j)))
        .map(FixedI48F16::exp);
    let numerator = mul(lambda, &term1_rhs);
    let denom = (FixedI48F16::from_num(2) * FixedI48F16::pi() * (sigma_sq + sigma_sq_j)).sqrt();

    let term1 = numerator / denom;

    let term2_rhs =
        (-diff2.map(|x| powi(x, 2)) / (FixedI48F16::from_num(2) * sigma_sq)).map(FixedI48F16::exp);
    let numerator = mul(FixedI48F16::from_num(1) - lambda, &term2_rhs);
    let denom = (FixedI48F16::from_num(2) * FixedI48F16::pi() * sigma_sq).sqrt();
    let term2 = numerator / denom;

    term1 + term2
}

pub fn neg_log_likelihood(
    params: &[FixedI48F16],
    pt: &DVector<FixedI48F16>,
    pt_1: &DVector<FixedI48F16>,
) -> FixedI48F16 {
    let pdf_vals = mrjpdf(params, pt, pt_1);
    -(pdf_vals
        .map(|x| x + FixedI48F16::from_num(1e-10))
        .map(|x| natural_log(x).unwrap())
        .sum())
}
