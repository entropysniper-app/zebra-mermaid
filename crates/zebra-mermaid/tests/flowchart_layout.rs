//! Layout-level differential test: replays the calls that Pele's flowchart and swimlane
//! renderers make to `compoundLayout`, `layered` (from inside `compoundLayout`) and
//! `laneLayout`, recorded with their inputs and outputs, and compares every coordinate the Rust
//! port computes with Pele's, exactly.
//!
//! The recordings are made by a private oracle bundle (see the flowchart port notes):
//! `node target/agents/layout-flowchart/oracle/dump.mjs <fixtures>... > <dump>.jsonl`. Set
//! `ZM_LAYOUT_DUMP` to a dump file (or several, separated by `:`); without it the test looks for
//! `target/agents/layout-flowchart/layout-dump-*.jsonl` and is skipped when there is none.

mod common;

use std::path::PathBuf;

use serde_json::Value;
use zebra_mermaid::layout::compound::{compound_layout, CEdge, CNode, Dir};
use zebra_mermaid::layout::lanes::{lane_layout, LANE_PAD};
use zebra_mermaid::layout::layered::{layered, Kind, LEdge, LNode, LayeredOptions};

fn num(v: &Value) -> f64 {
    common::number(v)
}

fn dir(v: &Value) -> Option<Dir> {
    match v.as_str() {
        Some("TB") => Some(Dir::TB),
        Some("BT") => Some(Dir::BT),
        Some("LR") => Some(Dir::LR),
        Some("RL") => Some(Dir::RL),
        _ => None,
    }
}

fn options(v: &Value) -> LayeredOptions {
    LayeredOptions {
        groups: None,
        order_only: false,
        node_sep: num(&v["nodeSep"]),
        edge_sep: num(&v["edgeSep"]),
        rank_sep: num(&v["rankSep"]),
        port_sep: num(&v["portSep"]),
        tracks: v["tracks"].as_bool().unwrap_or(false),
        straight: v["straight"].as_bool().unwrap_or(false),
        apart: v["apart"].as_bool().unwrap_or(false),
        head_room: if v["headRoom"].is_null() { None } else { Some(num(&v["headRoom"])) },
    }
}

fn kind(v: &Value) -> Kind {
    match v.as_u64() {
        Some(0) => Kind::Node,
        Some(1) => Kind::Dummy,
        Some(2) => Kind::Label,
        Some(3) => Kind::StartPort,
        Some(4) => Kind::EndPort,
        other => panic!("kind {other:?}"),
    }
}

fn lnodes(v: &Value) -> Vec<LNode> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|n| LNode {
            w: num(&n["w"]),
            h: num(&n["h"]),
            kind: kind(&n["kind"]),
            pin: num(&n["pin"]),
            hint: n.get("hint").map_or(f64::NAN, num),
            key: n.get("key").map_or(f64::NAN, num),
            span: num(&n["span"]),
            x: num(&n["x"]),
            y: num(&n["y"]),
            rank: num(&n["rank"]) as i32,
        })
        .collect()
}

fn ledges(v: &Value) -> Vec<LEdge> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|e| LEdge {
            tail: e["tail"].as_u64().unwrap() as usize,
            head: e["head"].as_u64().unwrap() as usize,
            minlen: num(&e["minlen"]),
            label_w: num(&e["labelW"]),
            label_h: num(&e["labelH"]),
            tail_dx: num(&e["tailDx"]),
            head_dx: num(&e["headDx"]),
            tail_fixed: e["tailFixed"].as_bool().unwrap(),
            head_fixed: e["headFixed"].as_bool().unwrap(),
            reversed: e["reversed"].as_bool().unwrap(),
            points: e["points"].as_array().unwrap().iter().map(num).collect(),
            label_x: num(&e["labelX"]),
            label_y: num(&e["labelY"]),
        })
        .collect()
}

fn cnodes(v: &Value) -> Vec<CNode> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|n| CNode {
            w: num(&n["w"]),
            h: num(&n["h"]),
            parent: num(&n["parent"]) as isize,
            is_group: n["isGroup"].as_bool().unwrap(),
            dir: dir(&n["dir"]),
            pad_x: num(&n["padX"]),
            pad_top: num(&n["padTop"]),
            pad_bottom: num(&n["padBottom"]),
            min_w: num(&n["minW"]),
            seq: num(&n["seq"]),
            span: num(&n["span"]),
            x: num(&n["x"]),
            y: num(&n["y"]),
        })
        .collect()
}

fn cedges(v: &Value) -> Vec<CEdge> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|e| CEdge {
            src: e["src"].as_u64().unwrap() as usize,
            dst: e["dst"].as_u64().unwrap() as usize,
            minlen: num(&e["minlen"]),
            label_w: num(&e["labelW"]),
            label_h: num(&e["labelH"]),
            route: e["route"].as_array().unwrap().iter().map(num).collect(),
            label_x: num(&e["labelX"]),
            label_y: num(&e["labelY"]),
        })
        .collect()
}

fn same(a: f64, b: f64) -> bool {
    a == b || (a.is_nan() && b.is_nan())
}

fn check(what: &str, a: f64, b: f64) -> Result<(), String> {
    if same(a, b) {
        Ok(())
    } else {
        Err(format!("{what}: rust {a} != pele {b}"))
    }
}

fn check_list(what: &str, a: &[f64], b: &[f64]) -> Result<(), String> {
    if a.len() != b.len() {
        return Err(format!("{what}: length {} != {}\n  rust {a:?}\n  pele {b:?}", a.len(), b.len()));
    }
    for (i, (x, y)) in a.iter().zip(b).enumerate() {
        if !same(*x, *y) {
            return Err(format!("{what}[{i}]: rust {x} != pele {y}\n  rust {a:?}\n  pele {b:?}"));
        }
    }
    Ok(())
}

fn compare_call(call: &Value) -> Result<(), String> {
    let out = &call["out"];
    match call["fn"].as_str().unwrap() {
        "layered" => {
            let mut nodes = lnodes(&call["nodes"]);
            let mut edges = ledges(&call["edges"]);
            let r = layered(&mut nodes, &mut edges, &options(&call["opt"]));
            check("width", r.width, num(&out["result"]["width"]))?;
            check("height", r.height, num(&out["result"]["height"]))?;
            for (i, (n, e)) in nodes.iter().zip(lnodes(&out["nodes"])).enumerate() {
                if n.rank != e.rank {
                    return Err(format!("node {i} rank {} != {}", n.rank, e.rank));
                }
                check(&format!("node {i} x"), n.x, e.x)?;
                check(&format!("node {i} y"), n.y, e.y)?;
            }
            for (i, (a, b)) in edges.iter().zip(ledges(&out["edges"])).enumerate() {
                if a.reversed != b.reversed {
                    return Err(format!("edge {i} reversed {} != {}", a.reversed, b.reversed));
                }
                check(&format!("edge {i} tailDx"), a.tail_dx, b.tail_dx)?;
                check(&format!("edge {i} headDx"), a.head_dx, b.head_dx)?;
                check_list(&format!("edge {i} points"), &a.points, &b.points)?;
                check(&format!("edge {i} labelX"), a.label_x, b.label_x)?;
                check(&format!("edge {i} labelY"), a.label_y, b.label_y)?;
            }
            Ok(())
        }
        "compound" | "lanes" => {
            let mut nodes = cnodes(&call["nodes"]);
            let mut edges = cedges(&call["edges"]);
            let opt = options(&call["opt"]);
            let d = dir(&call["dir"]).unwrap();
            let r = if call["fn"] == "compound" {
                compound_layout(&mut nodes, &mut edges, d, &opt)
            } else {
                let pad = if call["pad"].is_null() { LANE_PAD } else { num(&call["pad"]) };
                lane_layout(&mut nodes, &mut edges, d, &opt, pad)
            };
            check("width", r.width, num(&out["result"]["width"]))?;
            check("height", r.height, num(&out["result"]["height"]))?;
            for (i, (n, e)) in nodes.iter().zip(cnodes(&out["nodes"])).enumerate() {
                check(&format!("node {i} x"), n.x, e.x)?;
                check(&format!("node {i} y"), n.y, e.y)?;
                check(&format!("node {i} w"), n.w, e.w)?;
                check(&format!("node {i} h"), n.h, e.h)?;
                check(&format!("node {i} padTop"), n.pad_top, e.pad_top)?;
                check(&format!("node {i} seq"), n.seq, e.seq)?;
                if n.parent != e.parent {
                    return Err(format!("node {i} parent {} != {}", n.parent, e.parent));
                }
            }
            for (i, (a, b)) in edges.iter().zip(cedges(&out["edges"])).enumerate() {
                if (a.src, a.dst) != (b.src, b.dst) {
                    return Err(format!("edge {i} ends {:?} != {:?}", (a.src, a.dst), (b.src, b.dst)));
                }
                check_list(&format!("edge {i} route"), &a.route, &b.route)?;
                check(&format!("edge {i} labelX"), a.label_x, b.label_x)?;
                check(&format!("edge {i} labelY"), a.label_y, b.label_y)?;
            }
            Ok(())
        }
        other => Err(format!("unknown call {other}")),
    }
}

fn dumps() -> Vec<PathBuf> {
    if let Ok(list) = std::env::var("ZM_LAYOUT_DUMP") {
        return list.split(':').map(PathBuf::from).collect();
    }
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/agents/layout-flowchart");
    let Ok(entries) = std::fs::read_dir(&dir) else { return Vec::new() };
    let mut out: Vec<PathBuf> = entries
        .map(|e| e.unwrap().path())
        .filter(|p| p.file_name().unwrap().to_string_lossy().starts_with("layout-dump-") && p.extension().is_some_and(|e| e == "jsonl"))
        .collect();
    out.sort();
    out
}

#[test]
fn layout_calls_match_pele() {
    let files = dumps();
    if files.is_empty() {
        eprintln!("no layout dumps; skipped");
        return;
    }
    let mut f = common::Failures::default();
    let mut counts = std::collections::BTreeMap::<String, (usize, usize)>::new();
    for file in files {
        for row in common::read_jsonl_path(&file) {
            for (k, call) in row["calls"].as_array().unwrap().iter().enumerate() {
                let name = call["fn"].as_str().unwrap().to_string();
                let result = compare_call(call);
                let entry = counts.entry(name.clone()).or_default();
                entry.1 += 1;
                if result.is_ok() {
                    entry.0 += 1;
                }
                f.check(result, || format!("{} {} call {k} ({name})", row["file"], row["id"]));
            }
        }
    }
    for (name, (ok, n)) in &counts {
        eprintln!("{name:>10}: {ok}/{n} calls identical to Pele");
    }
    f.assert_none("layout");
}
