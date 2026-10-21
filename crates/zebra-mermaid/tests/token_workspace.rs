//! Handwritten contracts for per-scan state, including dynamic tables beyond fixed grammars.
//! These supplementary tests are not ports of original Pele tests.

use std::cell::Cell;
use zebra_mermaid::diagrams::common::tokens::{tokenize, Pattern, Scanner, TokenType, EOF};
use zebra_mermaid::{DiagramType, ErrorCode};

thread_local! {
    static OPENERS: Cell<usize> = const { Cell::new(0) };
    static FACTORIES: Cell<usize> = const { Cell::new(0) };
    static SCANS: Cell<usize> = const { Cell::new(0) };
}

fn no_closer(_: &str, _: usize) -> Option<usize> {
    None
}

fn bang_opener(source: &str, at: usize) -> Option<usize> {
    OPENERS.set(OPENERS.get() + 1);
    source[at..].starts_with('!').then_some(at + 1)
}

fn failing_type() -> TokenType {
    TokenType::func("unclosed", no_closer, Some("!")).with_opener(bang_opener)
}

#[test]
fn disabled_patterns_keep_priority_and_errors_across_dynamic_table_boundaries() {
    const EDGES: &[(usize, &[usize])] = &[
        (1, &[1]),
        (62, &[62]),
        (63, &[63]),
        (64, &[64]),
        (65, &[65]),
        (127, &[127]),
        (128, &[128]),
        (255, &[255]),
        (256, &[256]),
        (1023, &[1023]),
    ];
    for kind in std::iter::once(0).chain(EDGES.iter().map(|&(kind, _)| kind)) {
        let mut types = vec![TokenType::literal("unused", "\0"); kind + 2];
        types[kind] = failing_type();
        types[kind + 1] = TokenType::literal("bang", "!");
        OPENERS.set(0);
        let tokens = tokenize("!!!", &types, DiagramType::Info).unwrap();
        assert_eq!(tokens.kinds, [kind + 1, kind + 1, kind + 1, EOF]);
        assert_eq!(tokens.starts, [0, 1, 2, 3]);
        assert_eq!(tokens.ends, [1, 2, 3, 3]);
        assert_eq!(OPENERS.get(), 1, "direct kind {kind}");

        OPENERS.set(0);
        let error = tokenize("!?", &types, DiagramType::Info).unwrap_err();
        assert_eq!(error.code, ErrorCode::Syntax);
        assert_eq!((error.line, error.column), (1, 2));
        assert_eq!(error.snippet, "!?\n-^");
        assert_eq!(error.parser_offset(), Some(1));
        assert_eq!(OPENERS.get(), 1, "failing kind {kind}");
    }
    for &(kind, longer) in EDGES {
        let mut types = vec![TokenType::literal("unused", "\0"); kind + 1];
        types[0] = TokenType::literal("bang", "!").with_longer(longer);
        types[kind] = failing_type();
        OPENERS.set(0);
        let tokens = tokenize("!!!", &types, DiagramType::Info).unwrap();
        assert_eq!(tokens.kinds, [0, 0, 0, EOF]);
        assert_eq!(tokens.starts, [0, 1, 2, 3]);
        assert_eq!(tokens.ends, [1, 2, 3, 3]);
        assert_eq!(OPENERS.get(), 1, "longer kind {kind}");
    }
}

fn alternating_scanner() -> Scanner {
    FACTORIES.set(FACTORIES.get() + 1);
    let mut calls = 0;
    Box::new(move |source, at| {
        calls += 1;
        SCANS.set(SCANS.get() + 1);
        (calls % 2 == 1 && source[at..].starts_with("aa")).then_some(at + 2)
    })
}

fn stateful_type() -> TokenType {
    TokenType {
        name: "alternating",
        pattern: Pattern::Scanner(alternating_scanner),
        hidden: false,
        opener: None,
        first: Some("a"),
        longer: &[],
    }
}

#[test]
fn scanner_factories_remain_lazy_and_state_is_local_to_each_scan() {
    let types = [stateful_type(), TokenType::literal("a", "a"), TokenType::literal("b", "b")];
    FACTORIES.set(0);
    SCANS.set(0);
    let tokens = tokenize("aaaaaa", &types, DiagramType::Info).unwrap();
    assert_eq!(tokens.kinds, [0, 1, 0, 1, EOF]);
    assert_eq!(tokens.starts, [0, 2, 3, 5, 6]);
    assert_eq!(tokens.ends, [2, 3, 5, 6, 6]);
    assert_eq!((FACTORIES.get(), SCANS.get()), (1, 4));

    let tokens = tokenize("aaa", &types, DiagramType::Info).unwrap();
    assert_eq!(tokens.kinds, [0, 1, EOF]);
    assert_eq!(tokens.starts, [0, 2, 3]);
    assert_eq!(tokens.ends, [2, 3, 3]);
    assert_eq!((FACTORIES.get(), SCANS.get()), (2, 6));

    assert_eq!(tokenize("bbb", &types, DiagramType::Info).unwrap().kinds, [2, 2, 2, EOF]);
    assert_eq!(tokenize("", &types, DiagramType::Info).unwrap().kinds, [EOF]);
    assert!(tokenize("?", &types, DiagramType::Info).is_err());
    assert_eq!((FACTORIES.get(), SCANS.get()), (2, 6));

    let longer = [TokenType::literal("a", "a").with_longer(&[1]), stateful_type()];
    let tokens = tokenize("aaaaaa", &longer, DiagramType::Info).unwrap();
    assert_eq!(tokens.kinds, [1, 0, 1, 0, EOF]);
    assert_eq!(tokens.starts, [0, 2, 3, 5, 6]);
    assert_eq!(tokens.ends, [2, 3, 5, 6, 6]);
    assert_eq!((FACTORIES.get(), SCANS.get()), (3, 10));
}
