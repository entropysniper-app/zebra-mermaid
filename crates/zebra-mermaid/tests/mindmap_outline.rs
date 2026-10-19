//! Mind maps and kanban boards share Pele's outline grammar. The harvested fixtures hold no input
//! that Pele rejects, so the errors of the grammar and of both databases are checked here
//! against what Pele 1.6.0 answers (computed with `tools/compat/oracle.mjs`).
//!
//! Set `ZM_OUTLINE_CASES` to a directory of `.jsonl` files in the harvest fixture format to
//! compare further generated inputs with Pele.

#![cfg(all(feature = "mindmap", feature = "kanban"))]

mod common;

use common::{compare_render, read_jsonl_path, Failures};
use zebra_mermaid::{render, ErrorCode, RenderOptions};

#[test]
fn generated_cases_match_pele() {
    let Ok(dir) = std::env::var("ZM_OUTLINE_CASES") else {
        eprintln!("set ZM_OUTLINE_CASES to compare generated outline inputs");
        return;
    };
    let mut files: Vec<_> =
        std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "jsonl")).collect();
    files.sort();
    let mut f = Failures::default();
    for path in files {
        let rows = read_jsonl_path(&path);
        let mut ok = 0;
        for row in &rows {
            let result = compare_render(row);
            if result.is_ok() {
                ok += 1;
            }
            f.check(result, || format!("{} {:?}", path.display(), row["source"].as_str().unwrap()));
        }
        eprintln!("{}: {ok}/{} identical to Pele", path.display(), rows.len());
    }
    f.assert_none("outline");
}

/// (source, Pele's error as code, message, line, column), or `None` where Pele draws the input.
#[allow(clippy::type_complexity)]
const ERRORS: &[(&str, Option<(&str, &str, usize, usize)>)] = &[
    ("mindmap\n", None),
    ("mindmap a\n", None),
    ("mindmap\n a\n  :::big\r\n", None),
    ("kanban\n", None),
    ("mindmap\n  a\n b\n", Some(("semantic", "There can be only one root. No parent could be found for (\"b\")", 0, 0))),
    ("mindmap\n a\n  b[x\n", Some(("syntax", "Parse error on line 5:\n\n^\nExpecting 'NODE_DEND', got '1'", 5, 1))),
    ("mindmap\n a\n  b{x}\n", Some(("syntax", "Lexical error on line 3. Unrecognized text.\n  b{x}\n---^", 3, 4))),
    (
        "mindmap\n a\n  b(x) c\n",
        Some(("syntax", "Parse error on line 3:\n  b(x) c\n------^\nExpecting 'SPACELINE', 'NL', 'EOF', got 'SPACELIST'", 3, 7)),
    ),
    ("mindmap\n a\n  ::icon(x\n", Some(("syntax", "Parse error on line 5:\n\n^\nExpecting 'SPACELINE', 'NL', 'EOF', got '1'", 5, 1))),
    ("mindmap\n 中文😀\n  b{x}\n", Some(("syntax", "Lexical error on line 3. Unrecognized text.\n  b{x}\n---^", 3, 4))),
    ("mindmap\n a\n  b[\"`x\n", Some(("syntax", "Parse error on line 5:\n\n^\nExpecting 'NODE_DEND', got '1'", 5, 1))),
    ("kanban\n a\n  b@{ shape: Rect }\n", Some(("semantic", "No such shape: Rect. Shape names should be lowercase.", 0, 0))),
    (
        "kanban\n a\n  b@{ label: [ }\n",
        Some(("syntax", "Metadata of \"b\" is not valid YAML. YAML: unexpected end of the stream within a flow collection", 0, 0)),
    ),
    ("kanban\n  a\n b\n c\n", Some(("semantic", "Items without section detected, found section (\"b\")", 0, 0))),
    ("kanban\n a\n  b@{ x^ }\n", Some(("syntax", "Lexical error on line 3. Unrecognized text.\n  b@{ x^ }\n-------^", 3, 8))),
    (
        "kanban\n a\n  b@{ label: x } y\n",
        Some((
            "syntax",
            "Parse error on line 3:\n  b@{ label: x } y\n----------------^\nExpecting 'SPACELINE', 'NL', 'EOF', 'SHAPE_DATA', got 'SPACELIST'",
            3,
            17,
        )),
    ),
    (
        "kanban\n a\n  b@{ label: \"x\n\n  y\" }\n  c@{ a: 1\n",
        Some(("syntax", "Parse error on line 8:\n\n^\nExpecting 'SPACELINE', 'NL', 'EOF', 'SHAPE_DATA', got '1'", 8, 1)),
    ),
];

#[test]
fn errors_match_pele() {
    for &(source, expected) in ERRORS {
        let result = render(source, &RenderOptions::default());
        match (result, expected) {
            (Ok(_), None) => {}
            (Err(e), Some((code, message, line, column))) => {
                let code = match code {
                    "syntax" => ErrorCode::Syntax,
                    _ => ErrorCode::Semantic,
                };
                assert_eq!((e.code, e.message.as_str(), e.line, e.column), (code, message, line, column), "{source:?}");
            }
            (Ok(_), Some(e)) => panic!("{source:?} rendered, Pele fails with {e:?}"),
            (Err(e), None) => panic!("{source:?} failed with {e:?}, Pele draws it"),
        }
    }
}
