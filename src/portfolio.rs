use ndarray::{Array1, Array2};
use ndarray_linalg::{Cholesky, UPLO};
use rayon::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg64;
use rand_distr::{Distribution, StandardNormal};
use std::sync::{Arc, Mutex};

use crate::risk_group;

/// Defines a portfolio that handles the simulation and correlation structure
pub struct Portfolio {
    /// Number of risk factors used in the portfolio model
    risk_factors: usize,
    /// Covariance matrix (positive semi-definite)
    cov: Array2<f64>,
    /// Cholesky decomposition of covariance matrix
    lower: Array2<f64>,
    /// Container of all risk groups
    risk_group: Vec<risk_group::RiskGroup>,
    /// Number of borrowers within portfolio
    num_borrower: usize,
}

impl Portfolio {
    /// Create new instance with covariance matrix. The cholesky decomposition will be computed
    pub fn new(cov: Array2<f64>) -> Self {
        let lower: Array2<f64> = cov.cholesky(UPLO::Lower).expect("No Cholesky decomposition possible");

        Self {
            risk_factors: cov.ncols(),
            cov: cov,
            lower: lower,
            risk_group: Vec::new(),
            num_borrower: 0,
        }
    }

    /// Add a risk group to the portfolio
    pub fn add_risk_group(&mut self, mut risk_group: risk_group::RiskGroup) {
        risk_group.set_norm(&self.cov);
        self.num_borrower += risk_group.num_borrower();
        self.risk_group.push(risk_group);
    }

    /// Get iterator over the risk groups
    pub fn iter_risk_group(&self) -> impl Iterator<Item = &risk_group::RiskGroup> {
        self.risk_group.iter()
    }

    /// Expected loss of portfolio
    pub fn expected_loss(&self) -> f64 {
        self.risk_group.iter().map(|rg| rg.iter_borrower().map(|borr| borr.expected_loss()).sum::<f64>()).sum()
    }

    /// Calculate a trial, i.e. simulate the factor model for all entities within the portfolio
    /// given the correlation structure. A random number generator is provided to sample the random
    /// variables. The function returns the loss per borrower.
    pub fn trial(&self, rng: &mut rand_pcg::Pcg64) -> Array1<f64> {
        let mut out_borr: Array1<f64> = Array1::zeros(self.num_borrower);

        // Random Number Generator
        let mut nrm_gen = StandardNormal.sample_iter(rng);

        // Generate systematic factors once
        let n = Array1::from_iter(nrm_gen.by_ref().take(self.risk_factors));
        let rf = self.lower.dot(&n);

        // Loop over portfolio
        let mut index: usize = 0;
        for rg in self.iter_risk_group() {
            // Risk Group idiosyncratic risk
            let e2 = nrm_gen.next().unwrap();

            for borr in rg.iter_borrower() {
                // Borrower idiosyncratic risk
                let e1 = nrm_gen.next().unwrap();

                // Systematic risk factor
                let y = borr.risk_factor(&rf);

                // Get correlated asset value
                let z = borr.asset_value(&y, &e1, &e2);

                // Migration
                let rating = borr.migration(&z);

                // Incurred Loss
                out_borr[index] = *borr.get_loss(&rating);

                index += 1;
            }
        }

        // return
        out_borr
    }

    /// Perform simulation of many trials in parallel
    pub fn simulate(&self, num_trials: usize, chunk_size: usize, seed: u64) -> (Vec<f64>, Array1<f64>) {
        // Create container of loss distribution
        let mut out = vec![0_f64; num_trials];
        let out_borr = Arc::new(Mutex::new(Array1::<f64>::zeros(self.num_borrower)));

        // Get number of chunks
        let num_chunks: usize = (num_trials + chunk_size - 1) / chunk_size;

        // Get streams
        let mut base_rng = Pcg64::seed_from_u64(seed);
        let streams: Vec<u128> = (0..num_chunks).map(|_| (base_rng.r#gen::<u128>() << 1) | 1).collect();

        // Loop
        out.par_chunks_mut(chunk_size).zip(streams.par_iter()).for_each(|(chunk, stream)| {
            let mut rng = Pcg64::new(seed as u128, *stream);

            // local container for sum of all chunks
            let mut loc_borr: Array1<f64> = Array1::zeros(self.num_borrower);

            for val in chunk.iter_mut() {
                let loss_borr = self.trial(&mut rng);
                *val = loss_borr.sum();
                loc_borr += &loss_borr;
            }

            // Lock and update output borr
            let mut share = out_borr.lock().unwrap();
            *share += &loc_borr;
        });

        // Get expected loss
        let mut el: Array1<f64> = Arc::try_unwrap(out_borr).expect("Multiple references encountered").into_inner().unwrap();
        el /= num_trials as f64;

        // return
        (out, el)
    }
}
