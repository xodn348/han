use proptest::prelude::*;

// The parser must never panic on arbitrary input. It is allowed (and
// expected) to return ParseError; what we are guarding against is unwinding
// or arithmetic overflow inside the parser itself.

proptest! {
    #[test]
    fn parser_does_not_panic_on_arbitrary_string(s in ".*") {
        let tokens = han::lexer::tokenize(&s);
        let _ = han::parser::parse(tokens);
    }

    #[test]
    fn parser_does_not_panic_on_arbitrary_bytes(bytes in proptest::collection::vec(any::<u8>(), 0..512)) {
        let s = String::from_utf8_lossy(&bytes);
        let tokens = han::lexer::tokenize(&s);
        let _ = han::parser::parse(tokens);
    }

    // Synthesize sequences that look superficially like Han programs by
    // splicing well-formed-ish keyword fragments with random punctuation.
    // The parser may reject them, but it must not crash.
    #[test]
    fn parser_does_not_panic_on_keyword_soup(
        seq in proptest::collection::vec(
            prop_oneof![
                Just("함수 f() {"),
                Just("반환 0"),
                Just("변수 x = 1"),
                Just("상수 y = 2"),
                Just("만약 참 이면 {"),
                Just("아니면 {"),
                Just("반복 i 안에서 0..3 {"),
                Just("출력(1)"),
                Just("}"),
                Just("(") , Just(")") , Just("[") , Just("]"),
                Just("+") , Just("-") , Just("*") , Just("/"),
                Just("=") , Just("==") , Just("!=") , Just("<="),
                Just(",") , Just(";") , Just("\n"),
            ],
            0..64,
        ),
    ) {
        let s = seq.join(" ");
        let tokens = han::lexer::tokenize(&s);
        let _ = han::parser::parse(tokens);
    }

    // Deeply nested braces / parens should be handled without stack overflow
    // up to a reasonable bound.
    #[test]
    fn parser_does_not_panic_on_nested_braces(depth in 0usize..32) {
        let mut s = String::new();
        for _ in 0..depth { s.push('{'); }
        for _ in 0..depth { s.push('}'); }
        let tokens = han::lexer::tokenize(&s);
        let _ = han::parser::parse(tokens);
    }
}
