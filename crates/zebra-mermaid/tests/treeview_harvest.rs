#![cfg(feature = "treeView")]

//! The treeView rows of the harvested sets, read one line at a time. Some of Pele's treeView
//! tests feed it text with lone UTF-16 surrogates (an emoji cut in half). Such text is not a
//! Rust string at all, so those lines cannot be rendered here and are counted as skipped;
//! `compat_harvest` cannot read the treeView files until its reader skips them too.

mod common;

use std::io::Read;
use std::path::{Path, PathBuf};

use common::{compare_render, Failures};
use serde_json::Value;

fn rows(path: &Path) -> (Vec<Value>, usize) {
    let text = if path.extension().is_some_and(|e| e == "gz") {
        let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let mut s = String::new();
        flate2::read::GzDecoder::new(&bytes[..]).read_to_string(&mut s).expect("gzip");
        s
    } else {
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    };
    let mut out = Vec::new();
    let mut skipped = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        match serde_json::from_str(line) {
            Ok(v) => out.push(v),
            // serde_json refuses a lone surrogate escape, which no Rust string can hold.
            Err(e) if line.contains("\\ud") || line.contains("\\uD") => {
                let _ = e;
                skipped += 1;
            }
            Err(e) => panic!("{}: {e}", path.display()),
        }
    }
    (out, skipped)
}

fn run(path: PathBuf, name: &str) {
    if !path.exists() {
        eprintln!("{} is missing; skipped", path.display());
        return;
    }
    let (rows, skipped) = rows(&path);
    let mut f = Failures::default();
    for row in &rows {
        f.check(compare_render(row), || {
            format!("{} {:?}", row["id"], row["source"].as_str().unwrap().chars().take(80).collect::<String>())
        });
    }
    eprintln!("{name}: {} rows, {} with lone surrogates skipped", rows.len(), skipped);
    f.assert_none(name);
}

#[test]
fn harvested_treeview_sample_matches_pele() {
    run(common::fixtures().join("harvest/treeView.jsonl.gz"), "treeView sample");
}

#[test]
fn harvested_treeview_full_set_matches_pele() {
    if std::env::var("ZM_FULL_HARVEST").is_err() {
        eprintln!("set ZM_FULL_HARVEST=1 to run the full harvested set");
        return;
    }
    run(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/compat/harvest/fixtures/treeView.jsonl"), "treeView full");
}
