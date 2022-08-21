/*!
    Bench the individual algorithm for each line type (Line, Quad, Cube)
*/
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use easy_signed_distance_field::{sdf_generate, Line, vec2};

pub fn bench_line(c: &mut Criterion) {
    // Simple lines
    let lines = [
        Line::Line { start: vec2(0.5, 0.0), end: vec2(1.0, 1.0) },
        Line::Line { start: vec2(1.0, 1.0), end: vec2(0.0, 1.0) },
        Line::Line { start: vec2(0.0, 1.0), end: vec2(0.5, 0.0) },
    ];

    c.bench_function("lines 32px", |b| b.iter(|| 
        sdf_generate(
            32,
            32,
            0,
            5.0,
            black_box(&lines),
        )
    ));

    c.bench_function("lines 64px", |b| b.iter(|| 
        sdf_generate(
            64,
            64,
            0,
            5.0,
            black_box(&lines),
        )
    ));

    // Quad
    let view_box_width = 24.0;
    let view_box_height = 24.0;
    let mut lines = [
        Line::Quad { start: vec2(4.0, 10.0), end: vec2(20.0, 15.0), control: vec2(12.0, 0.0) },
        Line::Quad { start: vec2(20.0, 15.0), end: vec2(4.0, 10.0), control: vec2(12.0, 24.0) },
    ];

    // Normalize the lines values between 0.0 and 1.0
    for line in lines.iter_mut() {
        line.normalize(view_box_width, view_box_height);
    }

    c.bench_function("quad 32px", |b| b.iter(|| 
        sdf_generate(
            32,
            32,
            0,
            5.0,
            black_box(&lines),
        )
    ));

    c.bench_function("quad 64px", |b| b.iter(|| 
        sdf_generate(
            64,
            64,
            0,
            5.0,
            black_box(&lines),
        )
    ));
}

criterion_group!(benches, bench_line);
criterion_main!(benches);
