use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let cases = [
        "+80012340000",
        "+61406823897",
        "+611900123456",
        "+32474091150",
        "+34666777888",
        "+34612345678",
        "+441212345678",
        "+13459492311",
        "+16137827274",
        "+1 520 878 2491",
        "+1-520-878-2491",
    ];

    for case in cases {
        c.bench_with_input(BenchmarkId::new("parse", case), &case, |b, case| {
            b.iter(|| {
                let pn = black_box(case);
                phonenumber::parse(None, pn)
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
