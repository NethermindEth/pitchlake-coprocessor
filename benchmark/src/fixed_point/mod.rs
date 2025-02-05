use eyre::Result;
use nalgebra::{DVector, RealField};
use num_traits::Zero;
use simba::scalar::{ComplexField, FixedI44F20};

pub mod simulate_price;
pub use simulate_price::*;

pub type FixedPoint = FixedI44F20;

fn mul(lhs: FixedPoint, rhs: &DVector<FixedPoint>) -> DVector<FixedPoint> {
    let mut res = rhs.clone().into_owned();
    for rhs in res.as_mut_slice().iter_mut() {
        *rhs *= lhs
    }
    res
}

pub fn powi(x: FixedPoint, y: usize) -> FixedPoint {
    let mut result = FixedPoint::from_num(1);
    for _ in 0..y {
        result = result * x;
    }
    result
}

pub fn natural_log(x: FixedPoint) -> Result<FixedPoint> {
    if x <= FixedPoint::zero() {
        return Err(eyre::eyre!("Cannot take logarithm of non-positive number"));
    }
    let mut power = 0i32;
    let two = FixedPoint::from_num(2);
    let one = FixedPoint::from_num(1);
    let mut val = x;
    while val >= two {
        val = val / two;
        power += 1;
    }
    while val < one {
        val = val * two;
        power -= 1;
    }
    let base_ln = FixedPoint::ln_2() * FixedPoint::from_num(power);
    let frac = val - one;
    let frac_contribution = frac * FixedPoint::ln_2();
    Ok(base_ln + frac_contribution)
}

pub fn mrjpdf(
    params: &[FixedPoint],
    pt: &DVector<FixedPoint>,
    pt_1: &DVector<FixedPoint>,
) -> DVector<FixedPoint> {
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
        / (FixedPoint::from_num(2) * (sigma_sq + sigma_sq_j)))
        .map(FixedPoint::exp);
    let numerator = mul(lambda, &term1_rhs);
    let denom = (FixedPoint::from_num(2) * FixedPoint::pi() * (sigma_sq + sigma_sq_j)).sqrt();

    let term1 = numerator / denom;

    let term2_rhs =
        (-diff2.map(|x| powi(x, 2)) / (FixedPoint::from_num(2) * sigma_sq)).map(FixedPoint::exp);
    let numerator = mul(FixedPoint::from_num(1) - lambda, &term2_rhs);
    let denom = (FixedPoint::from_num(2) * FixedPoint::pi() * sigma_sq).sqrt();
    let term2 = numerator / denom;

    term1 + term2
}

pub fn neg_log_likelihood(
    params: &[FixedPoint],
    pt: &DVector<FixedPoint>,
    pt_1: &DVector<FixedPoint>,
) -> FixedPoint {
    let pdf_vals = mrjpdf(params, pt, pt_1);
    -(pdf_vals
        .map(|x| x + FixedPoint::from_num(1e-10))
        .map(|x| natural_log(x).unwrap())
        .sum())
}
