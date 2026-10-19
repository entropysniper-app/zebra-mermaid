//! Renders Pele's documentation corpus and compares each SVG, size, link list and error with
//! what Pele 1.6.0 gives (`tests/fixtures/corpus/*.jsonl`, from `tools/compat/gen-fixtures.mjs`).
//! Types this build does not render must report `unsupported-diagram`.

mod common;

use std::collections::BTreeMap;

use common::{compare_render, read_jsonl, render_options, Failures};
use zebra_mermaid::{render, type_enabled, DiagramType};

#[test]
fn corpus_matches_pele_for_enabled_types() {
    let dir = common::fixtures().join("corpus");
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
    files.sort();
    let mut f = Failures::default();
    let mut summary: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for path in files {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let enabled = match name.parse::<DiagramType>() {
            Ok(t) => type_enabled(t),
            // Inputs with no detected type: Rust must fail the same way.
            Err(()) => true,
        };
        for row in read_jsonl(&format!("corpus/{name}.jsonl")) {
            let entry = summary.entry(name.clone()).or_default();
            entry.1 += 1;
            if enabled {
                let result = compare_render(&row);
                if result.is_ok() {
                    entry.0 += 1;
                }
                f.check(result, || row["id"].as_str().unwrap().to_string());
            } else {
                // Not built: the type must be refused, not drawn some other way.
                let r = render(row["source"].as_str().unwrap(), &render_options(&row["options"], &row["icons"]));
                let refused = matches!(&r, Err(e) if e.code == zebra_mermaid::ErrorCode::UnsupportedDiagram);
                f.check(if refused { Ok(()) } else { Err("not refused as unsupported".into()) }, || {
                    row["id"].as_str().unwrap().to_string()
                });
            }
        }
    }
    for (name, (ok, n)) in &summary {
        let built = name.parse::<DiagramType>().map_or(true, type_enabled);
        if built {
            eprintln!("{name:>14}: {ok}/{n} identical to Pele");
        } else {
            eprintln!("{name:>14}: {n} refused as unsupported (not built)");
        }
    }
    f.assert_none("corpus");
}
