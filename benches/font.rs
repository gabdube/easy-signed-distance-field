/*!
    Bench the individual algorithm for each line type (Line, Quad, Cube)
*/
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use easy_signed_distance_field::Font;

pub fn bench_font(c: &mut Criterion) {
    use std::fs;

    let font_data = fs::read("./test_fixtures/Questrial-Regular.ttf").expect("Failed to read font file");
    let font = Font::from_bytes(font_data.as_slice(), Default::default()).expect("Failed to parse font file");

    c.bench_function("font character 64px", |b| b.iter(|| 
        font.sdf_generate(64.0, 2, 6.0, black_box('a')).unwrap()
    ));

    c.bench_function("alphabet 64px", |b| b.iter(|| 
        {
            for c in 'A'..='Z' {
                font.sdf_generate(64.0, 2, 6.0, black_box(c)).unwrap();
            }

            for c in 'a'..='z' {
                font.sdf_generate(64.0, 2, 6.0, black_box(c)).unwrap();
            }
        }
    ));

}

criterion_group!(benches, bench_font);
criterion_main!(benches);
