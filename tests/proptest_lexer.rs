use proptest::prelude::*;

proptest! {
    // The lexer must never panic on arbitrary input — no matter what bytes
    // we throw at it, it should return some token vector (possibly with
    // unknown/error tokens) without unwinding.
    #[test]
    fn lexer_does_not_panic_on_arbitrary_string(s in ".*") {
        let _ = han::lexer::tokenize(&s);
    }

    #[test]
    fn lexer_does_not_panic_on_arbitrary_bytes(bytes in proptest::collection::vec(any::<u8>(), 0..512)) {
        // Lossy UTF-8 conversion guarantees a valid &str input; any sequence
        // of bytes should still not crash the lexer.
        let s = String::from_utf8_lossy(&bytes);
        let _ = han::lexer::tokenize(&s);
    }

    #[test]
    fn lexer_does_not_panic_on_korean_keywords_with_garbage(
        prefix in "[\\u{AC00}-\\u{D7A3}]{0,8}",
        garbage in ".*",
        suffix in "[\\u{AC00}-\\u{D7A3}]{0,8}",
    ) {
        let s = format!("{}{}{}", prefix, garbage, suffix);
        let _ = han::lexer::tokenize(&s);
    }

    #[test]
    fn lexer_does_not_panic_on_numeric_soup(
        ints in proptest::collection::vec(any::<i32>(), 0..32),
        floats in proptest::collection::vec(any::<f32>(), 0..32),
    ) {
        let mut s = String::new();
        for i in ints {
            s.push_str(&i.to_string());
            s.push(' ');
        }
        for f in floats {
            s.push_str(&f.to_string());
            s.push(' ');
        }
        let _ = han::lexer::tokenize(&s);
    }
}
