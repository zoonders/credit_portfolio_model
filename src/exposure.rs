
/// Simple container of a single exposure and its valuations
pub struct Exposure {
    /// Valuations for each rating class
    valuation: Box<[f64]>,
}

impl Exposure {
    /// Create a new instance with its valuations
    pub fn new(valuation: Vec<f64>) -> Self {
        Self {
            valuation: valuation.into_boxed_slice(),
        }
    }

    /// Get valuation of a specified rating class by its index
    pub fn get_value(&self, index: &usize) -> &f64 {
        self.valuation.get(*index).expect("Index out of range")
    }

    /// Number of valuations, should be equal to number of rating classes
    pub fn num_values(&self) -> usize {
        self.valuation.len()
    }
}
