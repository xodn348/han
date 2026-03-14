use std::process::Command;

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
