use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dither_some::frame::Frame;

fn frame_get_benchmark(c: &mut Criterion) {
    let mut buf = vec![0u8; 1920 * 1080 * 3];
    let frame = Frame::new(1920, 1080, &mut buf);

    c.bench_function("frame_get", |b| {
        b.iter(|| frame.get_rgb(black_box(1), black_box(2)))
    });
}

fn frame_set_benchmark(c: &mut Criterion) {
    let mut buf = vec![0u8; 1920 * 1080 * 3];
    let mut frame = Frame::new(1920, 1080, &mut buf);

    c.bench_function("frame_set", |b| {
        b.iter(|| frame.set_rgb(black_box(1), black_box(2), black_box((1.0, 2.0, 3.0))))
    });
}

criterion_group!(frame_benchmark, frame_get_benchmark, frame_set_benchmark,);
criterion_main!(frame_benchmark);
