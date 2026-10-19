#![cfg(feature = "quadrantChart")]

//! The harvested quadrant chart inputs, compared with Pele like `compat_harvest`, but reading
//! the fixtures line by line, as two kinds of harvested line cannot reach this library:
//!
//! - inputs holding lone UTF-16 surrogates (`"quadrant-2 \ud83dGrowth"`), which no Rust
//!   string can hold;
//! - a `limit` option of `null`, which Pele compares as 0 and prints as "null" ("Diagram source
//!   is longer than the limit of null characters."), where `RenderOptions::limit` is a number
//!   or absent. This is the engine's option, not the diagram's.
//!
//! Those lines are counted and skipped; every other line must match.

mod common;

use std::io::Read;
use std::path::{Path, PathBuf};

use common::{compare_render, Failures};
use serde_json::Value;

fn lines(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    if path.extension().is_some_and(|e| e == "gz") {
        let mut s = String::new();
        flate2::read::GzDecoder::new(&bytes[..]).read_to_string(&mut s).expect("gzip");
        Some(s)
    } else {
        Some(String::from_utf8(bytes).expect("utf-8"))
    }
}

fn run(path: PathBuf) {
    let Some(text) = lines(&path) else {
        eprintln!("{} is missing; skipped", path.display());
        return;
    };
    let mut f = Failures::default();
    let mut unrepresentable = 0;
    let mut null_limit = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let row: Value = match serde_json::from_str(line) {
            Ok(row) => row,
            Err(e) => {
                assert!(line.contains("\\ud8") || line.contains("\\udc") || line.contains("\\ude"), "{e}: {line}");
                unrepresentable += 1;
                continue;
            }
        };
        if row["options"].get("limit").is_some_and(Value::is_null) {
            null_limit += 1;
            continue;
        }
        f.check(compare_render(&row), || {
            format!("{} {:?}", row["id"], row["source"].as_str().unwrap().chars().take(80).collect::<String>())
        });
    }
    eprintln!(
        "quadrantChart: {} compared, {} with lone surrogates and {} with a null limit skipped",
        f.checked, unrepresentable, null_limit
    );
    f.assert_none("quadrant harvest");
}

#[cfg(feature = "quadrantChart")]
#[test]
fn harvested_quadrant_sample_matches_pele() {
    run(common::fixtures().join("harvest/quadrantChart.jsonl.gz"));
}

#[cfg(feature = "quadrantChart")]
#[test]
fn harvested_quadrant_full_set_matches_pele() {
    if std::env::var("ZM_FULL_HARVEST").is_err() {
        eprintln!("set ZM_FULL_HARVEST=1 to run the full harvested set");
        return;
    }
    run(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/compat/harvest/fixtures/quadrantChart.jsonl"));
}
