use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ndarray::{Array1};
use ndarray_linalg::{Cholesky, UPLO};
use rand::SeedableRng;
use rand_pcg::Pcg64;
use rand_distr::{Distribution, StandardNormal};

use credit_portfolio_model::borrower::Borrower;

pub fn benchmark(c: &mut Criterion) {

    let mut rng = Pcg64::seed_from_u64(0);
    
    let n: usize = 100;

    let matrix = Array1::from_iter(StandardNormal.sample_iter(&mut rng).take(n * n)).into_shape_with_order((n, n)).unwrap();
    let cov = matrix.dot(&matrix.t());

    let lower = cov.cholesky(UPLO::Lower).expect("No Cholesky decomposition possible");

    let w = (0..n).map(|x| x as f64 / n as f64).collect();
    let r: usize = 1;
    let rho = 0.15;
    let eps = 0.85;
    let p_mig = vec![1. / n as f64; n];

    let mut borr = Borrower::new(w, r, rho, eps, p_mig);
    borr.set_norm(&cov);

    let rf = Array1::from_elem(n, 1.);

    let mut nrm_gen: rand_distr::DistIter<StandardNormal, _, f64> = StandardNormal.sample_iter(&mut rng);
    let x = Array1::from_iter(nrm_gen.by_ref().take(n));
   
    // called only once
    c.bench_function("portfolio::cholesky", |b| b.iter(|| cov.cholesky(UPLO::Lower).expect("No cholesky decomposition possible")));
    c.bench_function("borrower::set_norm", |b| b.iter(|| borr.set_norm(black_box(&cov))));
    c.bench_function("portfolio::setup_rng", |b| b.iter(|| Pcg64::seed_from_u64(black_box(0))));

    // called for each trial
    c.bench_function("portfolio::GetRand", |b| b.iter(|| Array1::from_iter(nrm_gen.by_ref().take(black_box(n)))));
    c.bench_function("portfolio::CorrelatedRand", |b| b.iter(|| lower.dot(black_box(&x))));

    // called for every borrower 
    c.bench_function("borrower::risk_factor", |b| b.iter(|| borr.risk_factor(black_box(&rf))));
    c.bench_function("borrower::asset_value", |b| b.iter(|| borr.asset_value(black_box(&0.), black_box(&-1.), black_box(&1.))));
    c.bench_function("borrower::migration", |b| b.iter(|| borr.migration(black_box(&0.))));
    c.bench_function("borrower::get_loss", |b| b.iter(|| borr.get_loss(black_box(&(n - 1)))));
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
