#![cfg(feature = "xychart")]

//! Temporary: compares rows of a JSONL file named by ZM_PROBE with Pele's answers.
mod common;
use common::{compare_render, Failures};

#[test]
fn probe() {
    let Ok(path) = std::env::var("ZM_PROBE") else { return };
    let text = std::fs::read_to_string(&path).unwrap();
    let mut f = Failures::default();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(row) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        f.check(compare_render(&row), || format!("{} {:?}", row["id"], row["source"].as_str().unwrap()));
    }
    f.assert_none("probe");
}
