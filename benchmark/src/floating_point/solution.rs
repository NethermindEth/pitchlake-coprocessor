/// Captures the essence of a function evaluation.
pub trait Evaluation<F> {
    /// Position `x` with the lowest corresponding value `f(x)`.
    fn position(&self) -> &[F];

    /// The actual value `f(x)`.
    fn value(&self) -> F;
}

/// A solution of a minimization run providing only the minimal information.
///
/// Each `Minimizer` might yield different types of solution structs which provide more
/// information.
#[derive(Debug, Clone)]
pub struct Solution<F> {
    /// Position `x` of the lowest corresponding value `f(x)` that has been found.
    pub position: Vec<F>,
    /// The actual value `f(x)`.
    pub value: F,
}

impl<F> Solution<F> {
    /// Creates a new `Solution` given the `position` as well as the corresponding `value`.
    pub fn new(position: Vec<F>, value: F) -> Solution<F> {
        Solution { position, value }
    }
}

impl<F> Evaluation<F> for Solution<F>
where
    F: Copy,
{
    fn position(&self) -> &[F] {
        &self.position
    }

    fn value(&self) -> F {
        self.value
    }
}
