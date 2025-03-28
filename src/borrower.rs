use statrs::distribution::{Normal, ContinuousCDF};
use ndarray::{Array1, Array2};
use crate::exposure;

/// Represents a borrower that is the atomic unit for rating migrations
pub struct Borrower {
    /// Dependency of the borrower to external weights, must match used covariance matrix
    risk_factor_weights: Array1<f64>,
    /// Current rating class as index
    rating: usize,
    /// Dependency on systematic/external factor
    rho: f64,
    /// Dependency on the risk group
    eps: f64,
    /// Migration probabilities into all possible classes
    p_mig: Array1<f64>,
    /// Migration probabilities transformed into a correpsonding threshold of a standard normal random
    /// variable
    c_mig: Box<[f64]>,
    /// Container of all exposures of the borrower
    exposures: Vec<exposure::Exposure>,
    /// Current valuation of all of the borrowers positions, based on exposures
    value: f64,
    /// Current valuations for all rating classes, based on exposures
    valuations: Array1<f64>,
    /// Losses derived from valuations and current valuation. Positive values indicate losses
    losses: Array1<f64>,
    /// Given a covariance matrix, this is the corresponding norm `N=\sqrt{\phi^T\dot\Sigma\dot\phi}`
    /// to result in a standard normal distributed random variable `z=\frac{\phi\dot\y}{N}`.
    norm: f64,
    /// Analytical calculation of expected loss given valuations and migration probabilities
    el: f64,
}

impl Borrower {
    /// Create new borrower given risk factor weights, current rating with migration probabilities
    /// and the dependency on the factor model. Thresholds of migrations will be calculated and
    /// empty containers created for exposures, valuations, losses, etc.
    pub fn new(risk_factor_weights: Vec<f64>, rating: usize, rho: f64, eps: f64, p_mig: Vec<f64>) -> Self {
        // Get migration thresholds
        // First, get cumulative probabilities
        let cum_p: Vec<f64> = p_mig.iter()
           .scan(0.0_f64, |sum, &x| {
               *sum += x;
               Some(*sum)
           }).collect();

        // Convert to normal distribution for all but last value
        let normal = Normal::new(0.0, 1.0).unwrap();
        let c_mig: Vec<f64> = cum_p.iter().take(cum_p.len() - 1)
           .map(|&p| normal.inverse_cdf(p))
           .collect();

        Self {
            risk_factor_weights: Array1::from(risk_factor_weights),
            rating: rating,
            rho: rho,
            eps: eps,
            p_mig: Array1::from(p_mig.clone()),
            c_mig: c_mig.into_boxed_slice(),
            exposures: Vec::new(),
            value: 0.0_f64,
            valuations: Array1::zeros(p_mig.len()),
            losses: Array1::zeros(p_mig.len()),
            norm: f64::NAN,
            el: 0.0_f64,
        }
    }

    /// Add an exposure to the borrower, the valuations and losses of the borrower will be updated
    pub fn add_exposure(&mut self, exposure: exposure::Exposure) {
        // Check if number of valuations of exposure is same as borrower expects
        if self.valuations.len() != exposure.num_values() {
            panic!("Exposure of length {} does not fit borrower with length {}", exposure.num_values(), self.valuations.len());
        }
        
        // add valuations to borrower valuations
        for (index, value) in self.valuations.iter_mut().enumerate() {
            *value += exposure.get_value(&index);
        }

        self.exposures.push(exposure);

        // Update current valuation
        self.value = self.valuations[self.rating];

        // Update losses
        self.losses.iter_mut().zip(self.valuations.iter()).for_each(|(a, &b)| *a = self.value - b);

        // Update expected loss
        self.el = self.expected_loss();
    }

    /// Set the relevant norm value given a covariance matrix to result in a standard normal
    /// distributed variable
    pub fn set_norm(&mut self, cov: &Array2<f64>) {
        self.norm = self.risk_factor_weights.dot(&cov.dot(&self.risk_factor_weights)).sqrt();
    }

    /// Given external risk factors, calculate the resulting standard normal variable of
    /// external/systematic factor
    pub fn risk_factor(&self, risk_factors: &Array1<f64>) -> f64 {
        risk_factors.dot(&self.risk_factor_weights) / self.norm
    }

    /// Apply the factor model formula given random variables for systematic, idiosyncratic and
    /// risk group random variables
    pub fn asset_value(&self, y: &f64, e1: &f64, e2: &f64) -> f64 {
        self.rho.sqrt() * y + (1. - self.rho).sqrt() * ((1. - self.eps).sqrt() * e1 + self.eps.sqrt() * e2)
    }

    /// Get the resulting rating grade given the result of the factor model
    pub fn migration(&self, z: &f64) -> usize {
        self.c_mig.binary_search_by(|a| a.partial_cmp(&z).expect("Only finite values should appear")).unwrap_or_else(|i| i)
    }

    /// Get the loss for a specified rating class (given by its index)
    pub fn get_loss(&self, index: &usize) -> &f64 {
        self.losses.get(*index).expect("Value not in range")
    }

    /// Calculate loss (analytically `EL=\sum_i{p_i\cdot l_i}`
    pub fn expected_loss(&self) -> f64 {
        self.p_mig.iter().zip(self.losses.iter()).map(|(p, l)| p * l).sum()
    }
}
