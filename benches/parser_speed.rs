use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::fs;

fn bench_parse_fibonacci(c: &mut Criterion) {
    let source =
        fs::read_to_string("examples/피보나치.hgl").expect("examples/피보나치.hgl should exist");

    c.bench_function("parse_fibonacci", |b| {
        b.iter(|| {
            let tokens = han::lexer::tokenize(black_box(&source));
            let _ = han::parser::parse(black_box(tokens)).unwrap();
        });
    });
}

fn bench_parse_factorial(c: &mut Criterion) {
    let source =
        fs::read_to_string("examples/팩토리얼.hgl").expect("examples/팩토리얼.hgl should exist");

    c.bench_function("parse_factorial", |b| {
        b.iter(|| {
            let tokens = han::lexer::tokenize(black_box(&source));
            let _ = han::parser::parse(black_box(tokens)).unwrap();
        });
    });
}

fn bench_parse_large_synthetic(c: &mut Criterion) {
    // Build a large synthetic source by repeating a fibonacci-like body.
    let unit = "함수 f(n: 정수) -> 정수 {\n    만약 n <= 1 이면 {\n        반환 n\n    }\n    반환 f(n - 1) + f(n - 2)\n}\n";
    let mut large = String::with_capacity(unit.len() * 200);
    for _ in 0..200 {
        large.push_str(unit);
    }

    c.bench_function("parse_large_synthetic_200x", |b| {
        b.iter(|| {
            let tokens = han::lexer::tokenize(black_box(&large));
            let _ = han::parser::parse(black_box(tokens)).unwrap();
        });
    });
}

fn bench_lex_only(c: &mut Criterion) {
    let source =
        fs::read_to_string("examples/할일목록.hgl").expect("examples/할일목록.hgl should exist");

    c.bench_function("lex_todo_list", |b| {
        b.iter(|| {
            let _ = han::lexer::tokenize(black_box(&source));
        });
    });
}

criterion_group!(
    benches,
    bench_parse_fibonacci,
    bench_parse_factorial,
    bench_parse_large_synthetic,
    bench_lex_only,
);
criterion_main!(benches);
