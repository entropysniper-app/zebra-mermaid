#![cfg(any(feature = "block", feature = "eventmodeling"))]

//! Compares the fixture rows of a local JSONL file with Pele, when `ZM_PROBE_ROWS` names one.
//! Used for rows regenerated outside the committed fixtures, such as harvested cases whose
//! `limit: Infinity` option JSON wrote as `null` (see the block and eventmodeling port report).

mod common;

use common::{compare_render, read_jsonl_path, Failures};

#[test]
fn probe_rows_match_pele() {
    let Ok(path) = std::env::var("ZM_PROBE_ROWS") else {
        eprintln!("set ZM_PROBE_ROWS to a JSONL file of fixture rows");
        return;
    };
    let mut f = Failures::default();
    for row in read_jsonl_path(std::path::Path::new(&path)) {
        f.check(compare_render(&row), || {
            format!("{} {:?}", row["id"], row["source"].as_str().unwrap().chars().take(80).collect::<String>())
        });
    }
    f.assert_none("probe");
}
