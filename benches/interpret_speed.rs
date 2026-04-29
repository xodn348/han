use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::fs;

fn run_program(source: &str) {
    han::interpreter::capture_start();
    let tokens = han::lexer::tokenize(source);
    let program = han::parser::parse(tokens).unwrap();
    han::interpreter::interpret(program).unwrap();
    let _ = han::interpreter::capture_flush();
}

fn bench_interpret_fibonacci(c: &mut Criterion) {
    let source =
        fs::read_to_string("examples/피보나치.hgl").expect("examples/피보나치.hgl should exist");

    c.bench_function("interpret_fibonacci", |b| {
        b.iter(|| run_program(black_box(&source)));
    });
}

fn bench_interpret_sum(c: &mut Criterion) {
    let source = fs::read_to_string("examples/합계.hgl").expect("examples/합계.hgl should exist");

    c.bench_function("interpret_sum_to_100", |b| {
        b.iter(|| run_program(black_box(&source)));
    });
}

fn bench_interpret_recursive_heavy(c: &mut Criterion) {
    // Heavier recursive workload — exercises Rc<RefCell> Environment scoping.
    let source = "함수 fib(n: 정수) -> 정수 {\n    만약 n <= 1 이면 {\n        반환 n\n    }\n    반환 fib(n - 1) + fib(n - 2)\n}\n함수 main() {\n    출력(fib(15))\n}\nmain()\n";

    c.bench_function("interpret_fib_15_recursive", |b| {
        b.iter(|| run_program(black_box(source)));
    });
}

fn bench_interpret_loop_heavy(c: &mut Criterion) {
    // Loop-heavy workload — exercises mutable variable scoping under Rc<RefCell>.
    let source = "함수 main() {\n    변수 합 = 0\n    반복 변수 i = 1; i <= 1000; i += 1 {\n        합 += i\n    }\n    출력(합)\n}\nmain()\n";

    c.bench_function("interpret_loop_sum_1000", |b| {
        b.iter(|| run_program(black_box(source)));
    });
}

criterion_group!(
    benches,
    bench_interpret_fibonacci,
    bench_interpret_sum,
    bench_interpret_recursive_heavy,
    bench_interpret_loop_heavy,
);
criterion_main!(benches);
