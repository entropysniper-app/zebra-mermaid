//! Runs the `zebra-mermaid` binary as a user would.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::Value;

const PIE: &str = "pie title Pets\n  \"Dogs\" : 386\n  \"Cats\" : 85\n";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_zebra-mermaid"))
}

fn run(args: &[&str], stdin: Option<&[u8]>) -> Output {
    let mut cmd = bin();
    cmd.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn");
    if let Some(input) = stdin {
        child.stdin.take().unwrap().write_all(input).unwrap();
    } else {
        drop(child.stdin.take());
    }
    child.wait_with_output().expect("wait")
}

fn code(o: &Output) -> i32 {
    o.status.code().unwrap_or(-1)
}

fn stdout(o: &Output) -> String {
    String::from_utf8(o.stdout.clone()).unwrap()
}

fn stderr(o: &Output) -> String {
    String::from_utf8(o.stderr.clone()).unwrap()
}

/// A fresh scratch directory under target/.
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("cli-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, text: &str) {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).unwrap();
    }
    fs::write(path, text).unwrap();
}

#[test]
fn renders_stdin_to_stdout_with_resolved_colors_by_default() {
    let o = run(&["render", "-"], Some(PIE.as_bytes()));
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    let svg = stdout(&o);
    assert!(svg.starts_with("<svg"));
    assert!(!svg.contains("var("), "CLI default is resolved colors");
    assert!(svg.contains("data-mermaid-canvas-background=\"opaque\""));
    assert!(stderr(&o).is_empty(), "nothing on stderr: {}", stderr(&o));
}

#[test]
fn css_mode_and_theme_options() {
    let o = run(&["render", "-", "--color-mode", "css"], Some(PIE.as_bytes()));
    assert_eq!(code(&o), 0);
    assert!(stdout(&o).contains("var(--pele-bg,#fff)"));
    let dark = run(&["render", "-", "--theme", "dark"], Some(PIE.as_bytes()));
    assert!(stdout(&dark).contains("#18181B"));
    let clear = run(&["render", "-", "--canvas-background", "transparent"], Some(PIE.as_bytes()));
    assert!(!stdout(&clear).contains("data-mermaid-canvas-background"));
    let bad = run(&["render", "-", "--theme", "resolved"], Some(PIE.as_bytes()));
    assert_eq!(code(&bad), 2);
    assert!(stderr(&bad).contains("--color-mode"));
}

#[test]
fn palette_file() {
    let dir = scratch("palette");
    let palette = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/research/zebra-theme-palette.example.json");
    let o = run(&["render", "-", "--palette", palette.to_str().unwrap()], Some(PIE.as_bytes()));
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    assert!(stdout(&o).contains("#2563EB"), "first series color");
    let bad = dir.join("bad.json");
    write(&bad, &fs::read_to_string(&palette).unwrap().replace("#2563EB", "#2563eb"));
    let o = run(&["render", "-", "--palette", bad.to_str().unwrap()], Some(PIE.as_bytes()));
    assert_eq!(code(&o), 2);
    assert!(stderr(&o).contains("invalid palette"), "{}", stderr(&o));
    let o = run(&["render", "-", "--palette", dir.join("missing.json").to_str().unwrap()], Some(PIE.as_bytes()));
    assert_eq!(code(&o), 5);
}

#[test]
fn output_file_is_not_replaced_without_force() {
    let dir = scratch("output");
    let input = dir.join("a.mmd");
    write(&input, PIE);
    let out = dir.join("a.svg");
    let o = run(&["render", input.to_str().unwrap(), "-o", out.to_str().unwrap()], None);
    assert_eq!(code(&o), 0, "{}", stderr(&o));
    assert!(fs::read_to_string(&out).unwrap().starts_with("<svg"));
    assert!(stdout(&o).is_empty());
    fs::write(&out, "keep").unwrap();
    let o = run(&["render", input.to_str().unwrap(), "-o", out.to_str().unwrap()], None);
    assert_eq!(code(&o), 5);
    assert!(stderr(&o).contains("--force"));
    assert_eq!(fs::read_to_string(&out).unwrap(), "keep");
    let o = run(&["render", input.to_str().unwrap(), "-o", out.to_str().unwrap(), "--force"], None);
    assert_eq!(code(&o), 0);
    assert!(fs::read_to_string(&out).unwrap().starts_with("<svg"));
    // No temporary files are left behind.
    let leftovers: Vec<_> =
        fs::read_dir(&dir).unwrap().filter(|e| e.as_ref().unwrap().file_name().to_string_lossy().ends_with(".tmp")).collect();
    assert!(leftovers.is_empty());
}

#[test]
fn json_errors_with_positions() {
    let src = "---\ntitle: x\n---\npie\n  \"中文\" 1\n";
    let o = run(&["check", "-", "--format", "json"], Some(src.as_bytes()));
    assert_eq!(code(&o), 3);
    let v: Value = serde_json::from_str(stdout(&o).trim()).unwrap();
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "syntax");
    assert_eq!(v["error"]["diagramType"], "pie");
    assert_eq!(v["error"]["mermaidType"], "pie");
    // Pele counts lines after the front matter; the source position counts them all.
    assert_eq!(v["error"]["line"], 2);
    assert_eq!(v["error"]["source"]["line"], 5);
    let byte = v["error"]["byteSpan"][0].as_u64().unwrap() as usize;
    assert_eq!(&src[byte..byte + 1], "1");
    assert_eq!(v["error"]["source"]["column"], 8);
}

#[test]
fn exit_codes() {
    let o = run(&["render", "-"], Some(b"notadiagram"));
    assert_eq!(code(&o), 3);
    assert!(stderr(&o).contains("unsupported-diagram"));
    let o = run(&["render", "-", "--limit", "5"], Some(PIE.as_bytes()));
    assert_eq!(code(&o), 4);
    let o = run(&["render", "-", "--max-input-bytes", "5"], Some(PIE.as_bytes()));
    assert_eq!(code(&o), 4);
    let o = run(&["render", "-"], Some(&[b'p', b'i', b'e', 0xff, b'\n']));
    assert_eq!(code(&o), 5);
    assert!(stderr(&o).contains("byte 3"), "{}", stderr(&o));
    let o = run(&["render", "/nonexistent/file.mmd"], None);
    assert_eq!(code(&o), 5);
    let o = run(&["render"], None);
    assert_eq!(code(&o), 2);
    let o = run(&["--version"], None);
    assert_eq!(code(&o), 0);
    assert!(stdout(&o).contains(&format!("Pele {}", zebra_mermaid::PELE_VERSION)));
}

#[test]
fn detect_and_json_render() {
    let o = run(&["detect", "-"], Some(PIE.as_bytes()));
    assert_eq!(stdout(&o).trim(), "pie");
    let o = run(&["render", "-", "--format", "json"], Some(PIE.as_bytes()));
    let v: Value = serde_json::from_str(&stdout(&o)).unwrap();
    assert_eq!(v["type"], "pie");
    assert_eq!(v["mermaidType"], "pie");
    assert!(v["svg"].as_str().unwrap().starts_with("<svg"));
}

#[test]
fn mermaid_type_in_render_check_and_errors() {
    let source = b"flowchart-elk TD\n  A --> B";
    for command in ["render", "check"] {
        for profile in [None, Some("--zebra")] {
            let mut args = vec![command, "-", "--format", "json"];
            args.extend(profile);
            let output = run(&args, Some(source));
            assert_eq!(code(&output), 0, "{}", stderr(&output));
            let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
            assert_eq!(value["type"], "flowchart");
            assert_eq!(value["mermaidType"], "flowchart-elk");
        }
    }
    for (source, args, status, diagram_type, mermaid_type) in [
        ("flowchart LR\n A -->", vec![], 3, Value::from("flowchart"), Value::from("flowchart-v2")),
        ("classDiagram\n class A", vec!["--output-limit", "10"], 4, Value::from("class"), Value::from("classDiagram")),
        ("not a diagram", vec![], 3, Value::Null, Value::Null),
        ("graph TD\n A", vec!["--limit", "5"], 4, Value::Null, Value::Null),
    ] {
        let mut command = vec!["render", "-", "--format", "json"];
        command.extend(args);
        let output = run(&command, Some(source.as_bytes()));
        assert_eq!(code(&output), status, "{}", stderr(&output));
        let value: Value = serde_json::from_str(&stdout(&output)).unwrap();
        assert_eq!(value["error"]["diagramType"], diagram_type);
        assert_eq!(value["error"]["mermaidType"], mermaid_type);
    }
}

#[test]
fn zebra_profile_ignores_max_width() {
    let a = run(&["render", "-", "--zebra"], Some(PIE.as_bytes()));
    let b = run(&["render", "-", "--zebra", "--max-width", "120"], Some(PIE.as_bytes()));
    assert_eq!(code(&a), 0);
    assert_eq!(stdout(&a), stdout(&b));
    let c = run(&["render", "-", "--max-width", "120"], Some(PIE.as_bytes()));
    assert_ne!(stdout(&a), stdout(&c), "the general profile fits the width");
}

#[test]
fn batch_keeps_relative_paths_and_reports_failures() {
    let dir = scratch("batch");
    let input = dir.join("in");
    write(&input.join("a.mmd"), PIE);
    write(&input.join("sub/b.mermaid"), "info");
    write(&input.join("sub/bad.mmd"), "pie\n  \"x\" 1\n");
    write(&input.join("notes.txt"), "ignored");
    let out = dir.join("out");
    let o = run(&["batch", input.to_str().unwrap(), "--out-dir", out.to_str().unwrap(), "--jobs", "2", "--format", "json"], None);
    assert_eq!(code(&o), 3, "{}", stdout(&o));
    assert!(out.join("a.svg").exists());
    assert!(out.join("sub/b.svg").exists());
    assert!(!out.join("sub/bad.svg").exists());
    let lines: Vec<Value> = stdout(&o).lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(lines.last().unwrap()["summary"]["ok"], 2);
    assert_eq!(lines.last().unwrap()["summary"]["failed"], 1);
    // A second run without --force refuses to replace the outputs.
    let o = run(&["batch", input.to_str().unwrap(), "--out-dir", out.to_str().unwrap()], None);
    assert_eq!(code(&o), 5);
    let o = run(&["batch", input.to_str().unwrap(), "--out-dir", out.to_str().unwrap(), "--force"], None);
    assert_eq!(code(&o), 3);
}

#[test]
fn batch_refuses_overlap_and_collisions() {
    let dir = scratch("overlap");
    let input = dir.join("in");
    write(&input.join("a.mmd"), PIE);
    let o = run(&["batch", input.to_str().unwrap(), "--out-dir", input.join("svg").to_str().unwrap()], None);
    assert_eq!(code(&o), 2);
    assert!(stderr(&o).contains("overlaps"));
    write(&input.join("a.mermaid"), PIE);
    let o = run(&["batch", input.to_str().unwrap(), "--out-dir", dir.join("out").to_str().unwrap()], None);
    assert_eq!(code(&o), 2);
    assert!(stderr(&o).contains("would both write"), "{}", stderr(&o));
}
