use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dither_some::dither;

fn dither_atkinson_benchmark(c: &mut Criterion) {
    let width = 1920;
    let height = 1080;
    let mut buf = vec![0u8; (width * height * 3) as usize];

    c.bench_function("dither_atkinson", |b| {
        b.iter(|| {
            dither::dither_frame_atkinson(
                black_box(width),
                black_box(height),
                black_box(&mut buf),
                black_box(2),
            );
        });
    });
}

fn dither_floyd_steinberg_benchmark(c: &mut Criterion) {
    let width = 1920;
    let height = 1080;
    let mut buf = vec![0u8; (width * height * 3) as usize];

    c.bench_function("dither_floyd_steinberg", |b| {
        b.iter(|| {
            dither::dither_frame_floyd_steinberg_color(
                black_box(width),
                black_box(height),
                black_box(&mut buf),
                black_box(2),
            );
        });
    });
}

fn custom_criterion() -> Criterion {
    Criterion::default().sample_size(20)
}

criterion_group! {
    name = dither_benchmark;
    config = custom_criterion();
    targets = dither_atkinson_benchmark, dither_floyd_steinberg_benchmark
}
criterion_main!(dither_benchmark);
