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

fn interpret(code: &str) -> String {
    han::interpreter::capture_start();

    let tokens = han::lexer::tokenize(code);
    let program = han::parser::parse(tokens).unwrap();
    han::interpreter::interpret(program).unwrap();

    han::interpreter::capture_flush().trim().to_string()
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
fn test_imyeon_conditional() {
    let out = interpret("변수 x = 10\n만약 x > 5 이면 {\n    출력(\"크다\")\n}\n");
    assert_eq!(out.trim(), "크다");
}

#[test]
fn test_const_immutability() {
    let out = interpret("상수 x = 42\n출력(x)\n");
    assert_eq!(out.trim(), "42");
}

#[test]
fn test_korean_logical_operators() {
    let out = interpret(
        "만약 참 그리고 거짓 이면 {\n    출력(1)\n} 아니면 {\n    출력(0)\n}\n만약 참 또는 거짓 이면 {\n    출력(1)\n}\n",
    );
    assert_eq!(out.trim(), "0\n1");
}

#[test]
fn test_string_interpolation_with_expression() {
    let out = interpret("출력(\"결과: ${1 + 2}\")\n");
    assert_eq!(out.trim(), "결과: 3");
}

#[test]
fn test_check_subcommand_success_and_failure() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let valid_path = std::env::temp_dir().join(format!("han_check_valid_{}.hgl", suffix));
    let invalid_path = std::env::temp_dir().join(format!("han_check_invalid_{}.hgl", suffix));

    fs::write(&valid_path, "변수 x: 정수 = 1\n출력(x)\n").expect("failed to write valid file");
    fs::write(&invalid_path, "변수 x: 정수 = \"안녕\"\n").expect("failed to write invalid file");

    let ok_output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "check",
            valid_path.to_str().expect("valid path should be utf8"),
        ])
        .output()
        .expect("failed to run hgl check on valid file");

    let fail_output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "check",
            invalid_path.to_str().expect("invalid path should be utf8"),
        ])
        .output()
        .expect("failed to run hgl check on invalid file");

    let _ = fs::remove_file(&valid_path);
    let _ = fs::remove_file(&invalid_path);

    assert!(
        ok_output.status.success(),
        "hgl check should pass on valid file: {}",
        String::from_utf8_lossy(&ok_output.stderr)
    );
    assert!(
        !fail_output.status.success(),
        "hgl check should fail on type mismatch"
    );
}

#[test]
fn test_init_subcommand_creates_scaffold_files() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let project_dir = std::env::temp_dir().join(format!("han_init_{}", suffix));

    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "init",
            project_dir
                .to_str()
                .expect("project path should be valid utf8"),
        ])
        .output()
        .expect("failed to execute hgl init");

    assert!(
        output.status.success(),
        "hgl init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(project_dir.join("main.hgl").exists());
    assert!(project_dir.join(".gitignore").exists());

    let _ = fs::remove_dir_all(&project_dir);
}

#[test]
fn test_contains_method() {
    let out = interpret("변수 arr = [1, 2, 3]\n만약 arr.포함(2) 이면 {\n    출력(\"있다\")\n}\n");
    assert_eq!(out.trim(), "있다");
}

#[test]
fn test_builtin_dot_product() {
    let out = interpret("출력(내적([1, 2, 3], [4, 5, 6]))\n");
    assert_eq!(out.trim(), "32");
}

#[test]
fn test_builtin_cross_product() {
    let out = interpret("출력(외적([1, 0, 0], [0, 1, 0]))\n");
    assert_eq!(out.trim(), "[0, 0, 1]");
}

#[test]
fn test_builtin_scalar_multiply() {
    let out = interpret("출력(스칼라곱([[1, 2], [3, 4]], 10))\n");
    assert_eq!(out.trim(), "[[10, 20], [30, 40]]");
}

#[test]
fn test_builtin_matrix_add() {
    let out = interpret("출력(행렬합([[1, 2], [3, 4]], [[5, 6], [7, 8]]))\n");
    assert_eq!(out.trim(), "[[6, 8], [10, 12]]");
}

#[test]
fn test_builtin_identity_matrix() {
    let out = interpret("출력(단위행렬(2))\n");
    assert_eq!(out.trim(), "[[1, 0], [0, 1]]");
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
        "시도 {\n    변수 result = 1 / 0\n    출력(111)\n} 처리(err) {\n    출력(222)\n}\n",
        "han_try_catch",
    );

    assert_eq!(out, "222");
}

#[test]
fn test_compiled_backend_string_method_length() {
    let out = build_and_run("출력(\"hello\".길이())\n", "han_string_len");

    assert_eq!(out, "5");
}

#[test]
fn test_compiled_backend_array_method_length() {
    let out = build_and_run(
        "변수 values: [정수] = [3, 4, 5]\n출력(values.길이())\n",
        "han_array_len",
    );

    assert_eq!(out, "3");
}

#[test]
fn test_compiled_backend_tuple_index_reads_stored_values() {
    let out = build_and_run(
        "변수 t = (11, 22, 33)\n출력(t.0)\n출력(t.2)\n",
        "han_tuple_index",
    );

    assert_eq!(out, "11\n33");
}

#[test]
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
맞춤 dir {
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
