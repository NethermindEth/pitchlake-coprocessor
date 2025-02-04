/// Captures the essence of a function evaluation.
pub trait Evaluation {
    /// Position `x` with the lowest corresponding value `f(x)`.
    fn position(&self) -> &[f64];

    /// The actual value `f(x)`.
    fn value(&self) -> f64;
}

/// A solution of a minimization run providing only the minimal information.
///
/// Each `Minimizer` might yield different types of solution structs which provide more
/// information.
#[derive(Debug, Clone)]
pub struct Solution {
    /// Position `x` of the lowest corresponding value `f(x)` that has been found.
    pub position: Vec<f64>,
    /// The actual value `f(x)`.
    pub value: f64,
}

impl Solution {
    /// Creates a new `Solution` given the `position` as well as the corresponding `value`.
    pub fn new(position: Vec<f64>, value: f64) -> Solution {
        Solution { position, value }
    }
}

impl Evaluation for Solution {
    fn position(&self) -> &[f64] {
        &self.position
    }

    fn value(&self) -> f64 {
        self.value
    }
}
