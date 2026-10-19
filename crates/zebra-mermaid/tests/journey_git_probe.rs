#![cfg(any(feature = "journey", feature = "gitGraph"))]

//! Extra differential cases for journey and gitGraph, generated locally with the oracle.
//! Runs only when `ZM_PROBE` names a harvest-format JSONL file.

mod common;

use common::{compare_render, read_jsonl_path, Failures};

#[test]
fn probe_cases_match_pele() {
    let Ok(path) = std::env::var("ZM_PROBE") else {
        eprintln!("set ZM_PROBE to a JSONL file to run the probe cases");
        return;
    };
    let mut f = Failures::default();
    for row in read_jsonl_path(std::path::Path::new(&path)) {
        let result = compare_render(&row);
        f.check(result, || format!("{} {:?} {}", row["id"], row["source"].as_str().unwrap(), row["options"]));
    }
    f.assert_none("probe");
}
