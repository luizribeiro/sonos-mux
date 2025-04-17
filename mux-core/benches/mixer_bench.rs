use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mux_core::mixer::{db_to_lin, lin_to_db};

fn bench_db_conversion(c: &mut Criterion) {
    c.bench_function("db_to_lin", |b| {
        b.iter(|| {
            for i in -100..100 {
                black_box(db_to_lin(i as f32 / 10.0));
            }
        });
    });

    c.bench_function("lin_to_db", |b| {
        b.iter(|| {
            for i in 1..100 {
                black_box(lin_to_db(i as f32 / 10.0));
            }
        });
    });
}

criterion_group!(benches, bench_db_conversion);
criterion_main!(benches);
