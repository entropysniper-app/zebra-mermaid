#![cfg(feature = "railroad")]

//! Railroad cases beyond the committed fixtures: the deep nesting Pele draws without a limit,
//! and, when `ZM_RAILROAD_CASES` names a JSONL file of cases with Pele's answers (written by a
//! generator against `tools/compat/oracle.mjs`, in the fixture format), each of those.

mod common;

use common::{compare_render, read_jsonl_path, Failures};
use zebra_mermaid::{render, RenderOptions};

#[test]
fn generated_cases_match_pele() {
    let Ok(path) = std::env::var("ZM_RAILROAD_CASES") else {
        eprintln!("set ZM_RAILROAD_CASES to a JSONL file of cases to compare them");
        return;
    };
    let mut f = Failures::default();
    for row in read_jsonl_path(std::path::Path::new(&path)) {
        let result = compare_render(&row);
        f.check(result, || format!("{} {:?}", row["id"], row["source"].as_str().unwrap()));
    }
    f.assert_none("railroad generated");
}

/// Pele parses brackets with explicit stacks and draws with loops, so it has no depth at
/// which it gives up (checked with the oracle to a million nested groups). Neither does the
/// port: deep nesting within the default source limit renders, in every notation.
#[test]
fn deep_nesting_renders_without_a_limit() {
    let n = 12_000;
    let options = RenderOptions { output_limit: Some(usize::MAX), ..Default::default() };
    let sources = [
        format!("railroad-ebnf-beta\nr = {}\"a\"{};\n", "(".repeat(n), ")".repeat(n)),
        format!("railroad-ebnf-beta\nr = {}\"a\"{};\n", "[".repeat(n), "]".repeat(n)),
        format!("railroad-abnf-beta\nr = {}\"a\"{};\n", "[".repeat(n), "]".repeat(n)),
        format!("railroad-peg-beta\nr <- {}\"a\"{};\n", "(".repeat(n), ")*".repeat(n)),
        format!("railroad-beta\nr = {}terminal(\"a\"){};\n", "optional(".repeat(n / 5), ")".repeat(n / 5)),
    ];
    for source in &sources {
        let r = render(source, &options).unwrap_or_else(|e| panic!("{}: {}", &source[..40], e.message));
        assert!(r.svg.ends_with("</svg>"));
    }
    // Plain groups draw one terminal however deep they are.
    let flat = render(&sources[0], &RenderOptions::default()).unwrap();
    let one = render("railroad-ebnf-beta\nr = \"a\";\n", &RenderOptions::default()).unwrap();
    assert_eq!(flat.svg, one.svg);
}
