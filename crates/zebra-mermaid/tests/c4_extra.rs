#![cfg(feature = "c4")]

//! Compares C4 renders with Pele for a local file of extra cases, in the fixture format, when
//! `ZM_C4_EXTRA` names one (for example inputs mutated from the harvested set and answered by
//! `tools/compat/oracle.mjs`). Without it the test does nothing.

mod common;

use common::{compare_render, read_jsonl_path, Failures};

#[test]
fn extra_c4_cases_match_pele() {
    let Ok(path) = std::env::var("ZM_C4_EXTRA") else {
        eprintln!("set ZM_C4_EXTRA to a .jsonl file of cases to compare");
        return;
    };
    let mut f = Failures::default();
    for row in read_jsonl_path(std::path::Path::new(&path)) {
        let result = compare_render(&row);
        f.check(result, || format!("{} {:?}", row["id"], row["source"].as_str().unwrap().chars().take(100).collect::<String>()));
    }
    f.assert_none("c4 extra");
}
