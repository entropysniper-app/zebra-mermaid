//! Renders the inputs harvested from Pele's own test suite (`tools/compat/harvest.mjs`) and
//! compares them with Pele. The committed sample in `tests/fixtures/harvest/*.jsonl.gz` always
//! runs; set `ZM_FULL_HARVEST=1` to run the full local set in `target/compat/harvest/fixtures/`.

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use common::{compare_render, read_jsonl_path, Failures};
use zebra_mermaid::{type_enabled, DiagramType};

fn run(dir: PathBuf, suffix: &str) {
    if !dir.exists() {
        eprintln!("{} is missing; skipped", dir.display());
        return;
    }
    let mut files: Vec<_> =
        std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.to_string_lossy().ends_with(suffix)).collect();
    files.sort();
    let mut f = Failures::default();
    let mut summary: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for path in files {
        let name = path.file_name().unwrap().to_str().unwrap().trim_end_matches(suffix).to_string();
        let built = name.parse::<DiagramType>().map_or(true, type_enabled);
        if !built {
            continue;
        }
        for row in read_jsonl_path(&path) {
            let entry = summary.entry(name.clone()).or_default();
            entry.1 += 1;
            let result = compare_render(&row);
            if result.is_ok() {
                entry.0 += 1;
            }
            f.check(result, || format!("{name} {} {:?}", row["id"], row["source"].as_str().unwrap().chars().take(80).collect::<String>()));
        }
    }
    for (name, (ok, n)) in &summary {
        eprintln!("{name:>14}: {ok}/{n} identical to Pele");
    }
    f.assert_none("harvest");
}

#[test]
fn harvested_sample_matches_pele() {
    run(common::fixtures().join("harvest"), ".jsonl.gz");
}

#[test]
fn harvested_full_set_matches_pele() {
    if std::env::var("ZM_FULL_HARVEST").is_err() {
        eprintln!("set ZM_FULL_HARVEST=1 to run the full harvested set");
        return;
    }
    run(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/compat/harvest/fixtures"), ".jsonl");
}
