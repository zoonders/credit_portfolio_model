use ndarray::Array2;

use crate::borrower;

/// Risk group which is a simple container of multiple borrowers sharing a common random variable
pub struct RiskGroup {
    /// Empty container
    borrower: Vec<borrower::Borrower>,
}

impl RiskGroup {
    /// Create instance with empty container
    pub fn new() -> Self {
        Self {
            borrower: Vec::new(),
        }
    }

    /// Add borrower to the risk group
    pub fn add_borrower(&mut self, borrower: borrower::Borrower) {
        self.borrower.push(borrower);
    }

    /// Get iterator of all borrowers
    pub fn iter_borrower(&self) -> impl Iterator<Item = &borrower::Borrower> {
        self.borrower.iter()
    }

    /// Get number of borrowers
    pub fn num_borrower(&self) -> usize {
        self.borrower.len()
    }

    /// Set the norm of all borrowers given a covariance matrix
    pub fn set_norm(&mut self, cov: &Array2<f64>) {
        self.borrower.iter_mut().for_each(|borr| borr.set_norm(cov));
    }
}
