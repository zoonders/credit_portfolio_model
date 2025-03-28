//! # Credit Portfolio Model
//! 
//! This crate provides a binary for simulation of a credit portfolio model.
//! The credit portfolio model is a factor-based model including asset correlations
//! and migration-mode changes in loan valuations.
//!
//! The binary of the code takes the input data
//! * Covariance Matrix
//! * Borrower information (rating, correlation to external factors and groups of connected
//! clients)
//! * Migration probabilities of borrowers
//! * Borrower dependency on external risk factors
//! * Exposure information
//! * Valuation of each exposure for all possible rating classes
//!
//! Based on this input, simulations are calculated and the total loss distribution is output

use clap::Parser;
use csv::{Reader, Writer};
use serde::Deserialize;
use std::path::Path;
use std::collections::HashMap;
use ndarray::Array2;
use statrs::statistics::{Data, Distribution, Median, OrderStatistics};
use chrono::Local;

mod portfolio;
mod risk_group;
mod borrower;
mod exposure;

// Arguments
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Input path
    #[arg(short, long)]
    input: String,

    /// Output path
    #[arg(short, long)]
    output: String,

    /// Number of simulated trials
    #[arg(short, long, default_value_t = 10)]
    num_trials: usize,

    /// Number of simulated trials per thread
    #[arg(short, long, default_value_t = 10_000)]
    chunk_size: usize,
}


// File input formats for serde
#[derive(Debug, Deserialize)]
struct CovarianceCell {
    risk_factor_1: usize,
    risk_factor_2: usize,
    correlation: f64,
}

#[derive(Debug, Deserialize)]
struct Borrower {
    borrower_id: String,
    risk_group: String,
    rating: usize,
    r2: f64,
    eps: f64,
}

#[derive(Debug, Deserialize)]
struct MigrationProb {
    borrower_id: String,
    rating: usize,
    probability: f64,
}

#[derive(Debug, Deserialize)]
struct RiskFactor {
    borrower_id: String,
    risk_factor: usize,
    weight: f64,
}

#[derive(Debug, Deserialize)]
struct Exposure {
    exposure_id: String,
    borrower_id: String,
    outstanding: f64,
}

#[derive(Debug, Deserialize)]
struct Valuation {
    exposure_id: String,
    rating: usize,
    valuation: f64,
}

fn read_input(path: &Path, risk_groups: &mut HashMap<String, Vec<Borrower>>, mig_probs: &mut HashMap<String, Vec<f64>>, risk_factors: &mut HashMap<String, Vec<f64>>, exposures: &mut HashMap<String, Vec<Exposure>>, valuations: &mut HashMap<String, Vec<f64>>) -> Array2<f64> {
    // Covariance
    let mut rdr = Reader::from_path(path.join("correlation_matrix.csv")).expect("Covariance file not found");

    let mut cells: Vec<CovarianceCell> = Vec::new();
    for result in rdr.deserialize() {
        let cell: CovarianceCell = result.unwrap();
        cells.push(cell);
    }

    let num_risk_factors = cells.iter().map(|x| x.risk_factor_1).max().expect("Only finite values") + 1;

    let mut cov: Array2<f64> = Array2::zeros((num_risk_factors, num_risk_factors));
    for cell in cells {
        cov[[cell.risk_factor_1, cell.risk_factor_2]] = cell.correlation;
    }
    
    // Borrower
    let mut rdr = Reader::from_path(path.join("borrower.csv")).expect("Borrower file not found");

    let mut rows: Vec<Borrower> = Vec::new();
    for result in rdr.deserialize() {
        let row: Borrower = result.unwrap();
        rows.push(row);
    }

    for borr in rows {
        risk_groups.entry(borr.risk_group.to_string()).or_insert(Vec::new()).push(borr);
    }

    // Transition probabilities
    let mut rdr = Reader::from_path(path.join("transition_probabilities.csv")).expect("Transition probability file not found");

    let mut rows: Vec<MigrationProb> = Vec::new();
    for result in rdr.deserialize() {
        let row: MigrationProb = result.unwrap();
        rows.push(row);
    }

    for row in rows {
        let entry = &mut mig_probs.entry(row.borrower_id.to_string()).or_insert(Vec::new());
        if entry.len() < row.rating + 1 {
            entry.resize(row.rating + 1, 0.);
        }
        entry[row.rating] = row.probability;
    }

    // Risk Factors
    let mut rdr = Reader::from_path(path.join("risk_factors.csv")).expect("Risk Factor file not found");

    let mut rows: Vec<RiskFactor> = Vec::new();
    for result in rdr.deserialize() {
        let row: RiskFactor = result.unwrap();
        rows.push(row);
    }

    for row in rows {
        let entry = &mut risk_factors.entry(row.borrower_id.to_string()).or_insert(vec![0.; num_risk_factors]);
        entry[row.risk_factor] = row.weight;
    }

    // Exposure
    let mut rdr = Reader::from_path(path.join("exposures.csv")).expect("Exposure file not found");

    let mut rows: Vec<Exposure> = Vec::new();
    for result in rdr.deserialize() {
        let row: Exposure = result.unwrap();
        rows.push(row);
    }

    for row in rows {
        exposures.entry(row.borrower_id.to_string()).or_insert(Vec::new()).push(row);
    }

    // Valuations
    let mut rdr = Reader::from_path(path.join("valuations.csv")).expect("Valuation file not found");

    let mut rows: Vec<Valuation> = Vec::new();
    for result in rdr.deserialize() {
        let row: Valuation = result.unwrap();
        rows.push(row);
    }

    for row in rows {
        let entry = &mut valuations.entry(row.exposure_id.to_string()).or_insert(Vec::new());
        if entry.len() < row.rating + 1 {
            entry.resize(row.rating + 1, 0.);
        }
        entry[row.rating] = row.valuation;
    }

    // return
    cov
}

fn main() {
    let args = Args::parse();

    let path = Path::new(&args.input);

    // Empty container of input data
    let mut risk_groups: HashMap<String, Vec<Borrower>> = HashMap::new();
    let mut mig_probs: HashMap<String, Vec<f64>> = HashMap::new();
    let mut risk_factors: HashMap<String, Vec<f64>> = HashMap::new();
    let mut exposures: HashMap<String, Vec<Exposure>> = HashMap::new();
    let mut valuations: HashMap<String, Vec<f64>> = HashMap::new();
    
    // Fill containers and get covariance matrix
    let cov = read_input(&path, &mut risk_groups, &mut mig_probs, &mut risk_factors, &mut exposures, &mut valuations);

    // Initialize
    let mut pf = portfolio::Portfolio::new(cov);

    for (_rg, borr_list) in risk_groups.drain() {
        let mut rg = risk_group::RiskGroup::new();

        for borr in borr_list {
            let prob = mig_probs.remove(&borr.borrower_id).expect("Probability not found");
            let rf = risk_factors.remove(&borr.borrower_id).expect("Risk Factor not found");
            let exp_list = exposures.remove(&borr.borrower_id).expect("Exposure List not found");

            let mut borr = borrower::Borrower::new(rf, borr.rating, borr.r2, borr.eps, prob);

            for exp in exp_list {
                let val = valuations.remove(&exp.exposure_id).expect("Valuation not found");

                let exp = exposure::Exposure::new(val);

                borr.add_exposure(exp);
            }

            rg.add_borrower(borr);
        }

        pf.add_risk_group(rg);
    }

    // Do simulation
    let start = Local::now();
    println!("Finished initialization {}", start.format("%Y-%m-%d %H:%M:%S"));

    // Simulation
    let (loss, el) = pf.simulate(args.num_trials, args.chunk_size, 0);
    
    let elapsed = Local::now() - start;
    println!("Done after {:.3} s", elapsed.num_milliseconds() as f64 / 1000.);

    let mut out = Data::new(loss);
    println!("Exp Loss:     {:15.2}", pf.expected_loss());
    println!("Exp Loss Sim: {:15.2}", el.sum());
    println!("Mean:         {:15.2}", out.mean().unwrap());
    println!("Median:       {:15.2}", out.median());
    println!("(90.0%):      {:15.2}", out.quantile(0.900));
    println!("(99.0%):      {:15.2}", out.quantile(0.990));
    println!("(99.9%):      {:15.2}", out.quantile(0.999));

    // Output
    let outpath = Path::new(&args.output);

    // Loss distribution
    let mut writer = Writer::from_path(outpath.join("loss_distribution.csv")).expect("Output path not found");
    writer.write_record(vec!["Loss"]).unwrap();
    out.iter().for_each(|row| writer.write_record(vec![row.to_string()]).unwrap());
}
