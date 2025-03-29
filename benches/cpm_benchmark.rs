use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ndarray::{Array1, Array2};
use credit_portfolio_model::borrower::Borrower;

pub fn borrower_benchmark(c: &mut Criterion) {

    let n: usize = 100;

    let matrix = Array2::from_shape_fn((n, n), |(i, j)| (i + j) as f64);
    let cov = matrix.dot(&matrix.t());

    let w = (0..n).map(|x| x as f64 / n as f64).collect();
    let r: usize = 1;
    let rho = 0.15;
    let eps = 0.85;
    let p_mig = vec![1. / n as f64; n];

    let mut borr = Borrower::new(w, r, rho, eps, p_mig);
    borr.set_norm(&cov);

    let rf = Array1::from_elem(n, 1.);
    
    c.bench_function("borrower::set_norm", |b| b.iter(|| borr.set_norm(black_box(&cov))));
    c.bench_function("borrower::risk_factor", |b| b.iter(|| borr.risk_factor(black_box(&rf))));
    c.bench_function("borrower::asset_value", |b| b.iter(|| borr.asset_value(black_box(&0.), black_box(&-1.), black_box(&1.))));
    c.bench_function("borrower::migration", |b| b.iter(|| borr.migration(black_box(&0.))));
    c.bench_function("borrower::get_loss", |b| b.iter(|| borr.get_loss(black_box(&(n - 1)))));
}

criterion_group!(benches, borrower_benchmark);
criterion_main!(benches);
