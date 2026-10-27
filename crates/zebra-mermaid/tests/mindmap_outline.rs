//! Mind maps and kanban boards share Pele's outline grammar. The harvested fixtures hold no input
//! that Pele rejects, so the errors of the grammar and of both databases are checked here
//! against the pinned Pele reference (computed with `tools/compat/gen-outline-errors.mjs`).
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

/// Each original error assertion, refreshed by the fixed upstream Node implementation.
#[test]
fn errors_match_pele() {
    let rows = common::read_jsonl("unit/outline-errors.jsonl");
    assert_eq!(rows.len(), 17);
    for row in rows {
        let source = row["source"].as_str().unwrap();
        let expected = &row["expect"];
        let result = render(source, &RenderOptions::default());
        match result {
            Ok(_) => assert!(expected.is_null(), "{source:?} rendered, Pele fails with {expected}"),
            Err(e) => {
                assert!(!expected.is_null(), "{source:?} failed with {e:?}, Pele draws it");
                let code = match expected["code"].as_str().unwrap() {
                    "syntax" => ErrorCode::Syntax,
                    _ => ErrorCode::Semantic,
                };
                assert_eq!(
                    (e.code, e.message.as_str(), e.line, e.column),
                    (
                        code,
                        expected["message"].as_str().unwrap(),
                        expected["line"].as_u64().unwrap() as usize,
                        expected["column"].as_u64().unwrap() as usize
                    ),
                    "{source:?}"
                );
            }
        }
    }
}
