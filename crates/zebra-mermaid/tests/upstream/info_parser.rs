//! 1:1 ports of Pele tests/compat/info/info.test.ts and info.spec.ts.
//! Original tests: Mermaid (c) 2014–2022 Knut Sveidqvist, MIT.
use zebra_mermaid::diagrams::info::{parse_info, InfoAst};
#[test]
fn parameter_00() {
    // Successful typed InfoAst is the Rust equivalent of no errors/alternatives and $type == Info.$type.
    let _: InfoAst = parse_info("info").unwrap();
}
#[test]
fn parameter_01() {
    // Successful typed InfoAst is the Rust equivalent of no errors/alternatives and $type == Info.$type.
    let _: InfoAst = parse_info("\n    info").unwrap();
}
#[test]
fn parameter_02() {
    // Successful typed InfoAst is the Rust equivalent of no errors/alternatives and $type == Info.$type.
    let _: InfoAst = parse_info("info\n    ").unwrap();
}
#[test]
fn parameter_03() {
    // Successful typed InfoAst is the Rust equivalent of no errors/alternatives and $type == Info.$type.
    let _: InfoAst = parse_info("\n    info\n    ").unwrap();
}
#[test]
fn parameter_04() {
    // Successful typed InfoAst is the Rust equivalent of no errors/alternatives and $type == Info.$type.
    let _: InfoAst = parse_info("info showInfo").unwrap();
}
#[test]
fn parameter_05() {
    // Successful typed InfoAst is the Rust equivalent of no errors/alternatives and $type == Info.$type.
    let _: InfoAst = parse_info("info showInfo\n    ").unwrap();
}
#[test]
fn parameter_06() {
    // Successful typed InfoAst is the Rust equivalent of no errors/alternatives and $type == Info.$type.
    let _: InfoAst = parse_info("\n    info showInfo").unwrap();
}
#[test]
fn parameter_07() {
    // Successful typed InfoAst is the Rust equivalent of no errors/alternatives and $type == Info.$type.
    let _: InfoAst = parse_info("info\n    showInfo").unwrap();
}
#[test]
fn parameter_08() {
    // Successful typed InfoAst is the Rust equivalent of no errors/alternatives and $type == Info.$type.
    let _: InfoAst = parse_info("info\n    showInfo\n    ").unwrap();
}
#[test]
fn parameter_09() {
    // Successful typed InfoAst is the Rust equivalent of no errors/alternatives and $type == Info.$type.
    let _: InfoAst = parse_info("\n    info\n    showInfo\n    ").unwrap();
}
#[test]
fn parameter_10() {
    // Successful typed InfoAst is the Rust equivalent of no errors/alternatives and $type == Info.$type.
    let _: InfoAst = parse_info("\n    info\n    showInfo").unwrap();
}
#[test]
fn parameter_11() {
    // Successful typed InfoAst is the Rust equivalent of no errors/alternatives and $type == Info.$type.
    let _: InfoAst = parse_info("\n    info showInfo\n    ").unwrap();
}
#[test]
fn definition() {
    parse_info("info").unwrap();
}
#[test]
fn definition_with_show_info() {
    parse_info("info showInfo").unwrap();
}
#[test]
#[ignore = "Pele upstream skip: Langium lexer error message differs; not a pass"]
fn unsupported_grammar_first() {
    assert!(parse_info("info unsupported")
        .unwrap_err()
        .message
        .contains("Parsing failed: Lexer error on line 1, column 6: unexpected character: ->u<- at offset: 5, skipped 11 characters."));
}
#[test]
#[ignore = "Pele upstream skip: duplicate test preserved separately; not a pass"]
fn unsupported_grammar_second() {
    assert!(parse_info("info unsupported")
        .unwrap_err()
        .message
        .contains("Parsing failed: Lexer error on line 1, column 6: unexpected character: ->u<- at offset: 5, skipped 11 characters."));
}
