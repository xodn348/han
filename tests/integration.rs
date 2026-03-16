use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_interpret(file: &str) -> String {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "interpret", file])
        .output()
        .expect("failed to execute hgl");

    assert!(
        output.status.success(),
        "hgl interpret {} failed: {}",
        file,
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout)
        .expect("invalid utf8")
        .trim()
        .to_string()
}

static BUILD_LOCK: Mutex<()> = Mutex::new(());

fn build_and_run(source: &str, stem_prefix: &str) -> String {
    let _lock = BUILD_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let stem = format!("{}_{}", stem_prefix, unique_suffix);
    let source_path = std::env::temp_dir().join(format!("{}.hgl", stem));
    let output_path = PathBuf::from(format!("./{}", stem));

    fs::write(&source_path, source).expect("failed to write temp source");

    let build_output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "build",
            source_path
                .to_str()
                .expect("temp path should be valid utf8"),
        ])
        .output()
        .expect("failed to execute hgl build");

    assert!(
        build_output.status.success(),
        "hgl build failed: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    let run_output = Command::new(&output_path)
        .output()
        .expect("failed to execute built binary");

    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);

    assert!(
        run_output.status.success(),
        "built binary failed: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );

    String::from_utf8(run_output.stdout)
        .expect("invalid utf8")
        .trim()
        .to_string()
}

#[test]
fn test_hello() {
    let out = run_interpret("examples/안녕.hgl");
    assert_eq!(out, "안녕하세요, 세계!");
}

#[test]
fn test_fibonacci() {
    let out = run_interpret("examples/피보나치.hgl");
    assert_eq!(out, "55");
}

#[test]
fn test_factorial() {
    let out = run_interpret("examples/팩토리얼.hgl");
    assert_eq!(out, "3628800");
}

#[test]
fn test_sum() {
    let out = run_interpret("examples/합계.hgl");
    assert_eq!(out, "5050");
}

#[test]
fn test_even_odd() {
    let out = run_interpret("examples/짝홀.hgl");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 10);
    assert_eq!(lines[0], "홀수");
    assert_eq!(lines[1], "짝수");
    assert_eq!(lines[9], "짝수");
}

#[test]
fn test_compiled_backend_range_for_in_outputs_sequence() {
    let out = build_and_run("반복 i 안에서 0..5 {\n    출력(i)\n}\n", "han_range_for_in");

    assert_eq!(out, "0\n1\n2\n3\n4");
}

#[test]
fn test_compiled_backend_array_for_in_outputs_items() {
    let out = build_and_run(
        "변수 values = [3, 4, 5]\n반복 value 안에서 values {\n    출력(value)\n}\n",
        "han_array_for_in",
    );

    assert_eq!(out, "3\n4\n5");
}

#[test]
fn test_compiled_backend_try_catch_handles_division_by_zero() {
    let out = build_and_run(
        "시도 {\n    변수 result = 1 / 0\n    출력(111)\n} 실패(err) {\n    출력(222)\n}\n",
        "han_try_catch",
    );

    assert_eq!(out, "222");
}

#[test]
#[ignore = "codegen: method call codegen not yet implemented"]
fn test_compiled_backend_string_method_length() {
    let out = build_and_run("출력(\"hello\".길이())\n", "han_string_len");

    assert_eq!(out, "5");
}

#[test]
#[ignore = "codegen: method call codegen not yet implemented"]
fn test_compiled_backend_array_method_length() {
    let out = build_and_run(
        "변수 values: [정수] = [3, 4, 5]\n출력(values.길이())\n",
        "han_array_len",
    );

    assert_eq!(out, "3");
}

#[test]
#[ignore = "codegen: method call codegen not yet implemented"]
fn test_compiled_backend_struct_impl_method_call() {
    let out = build_and_run(
        "구조 Rect { width: 정수, height: 정수 }\n구현 Rect {\n    함수 area(자신: Rect) -> 정수 {\n        반환 자신.width * 자신.height\n    }\n}\n변수 rect: Rect = Rect { width: 2, height: 3 }\n출력(rect.area())\n",
        "han_struct_method",
    );

    assert_eq!(out, "6");
}

#[test]
#[ignore = "codegen: enum variant IR generation in progress"]
fn test_compiled_backend_enum_match_branches_by_variant_tag() {
    let out = build_and_run(
        "열거 Direction { Up, Down }
변수 dir = Direction::Down
맞춰 dir {
    Up => 출력(11)
    Down => 출력(22)
    _ => 출력(33)
}
",
        "han_enum_match",
    );

    assert_eq!(out, "22");
}

#[test]
#[ignore = "codegen: Korean identifiers in LLVM IR not yet supported"]
fn test_compiled_backend_lambda_outputs_value() {
    let out = build_and_run(
        "변수 두배 = 함수(x: 정수) {
    반환 x * 2
}
출력(두배(5))
",
        "han_lambda_basic",
    );

    assert_eq!(out, "10");
}

#[test]
#[ignore = "codegen: Korean identifiers in LLVM IR not yet supported"]
fn test_compiled_backend_closure_captures_outer_variable() {
    let out = build_and_run(
        "변수 배수 = 3
변수 곱하기 = 함수(x: 정수) {
    반환 x * 배수
}
출력(곱하기(4))
",
        "han_lambda_capture",
    );

    assert_eq!(out, "12");
}
