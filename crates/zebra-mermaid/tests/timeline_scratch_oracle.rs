#![cfg(any(feature = "timeline", feature = "ishikawa", feature = "cynefin"))]

//! Scratch harness (temporary): compares rows of a JSONL file named by ZM_TIC_JSONL.
mod common;
use common::{compare_render, read_jsonl_path, Failures};

#[test]
fn scratch_rows_match_pele() {
    let Ok(path) = std::env::var("ZM_TIC_JSONL") else { return };
    let mut f = Failures::default();
    for row in read_jsonl_path(std::path::Path::new(&path)) {
        let result = compare_render(&row);
        f.check(result, || format!("{} {:?}", row["id"], row["source"].as_str().unwrap()));
    }
    f.assert_none("scratch");
}
