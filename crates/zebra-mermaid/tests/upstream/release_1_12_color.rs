use super::*;
fn group(s: &str, id: &str) -> String {
    captures(s, &format!(r#"data-id="{}"[^>]*>(.*?)</g>"#, regex::escape(id)))[1].clone()
}
fn nested_group(s: &str, id: &str) -> String {
    captures(s, &format!(r#"data-id="{}"[^>]*>(.*?)</g></g>"#, regex::escape(id)))[1].clone()
}
fn casing(s: &str) -> String {
    captures(s, r#"<g class="pele-casings"[^>]*>(.*?)</g>"#)[1].clone()
}
fn edge(s: &str, id: &str) -> String {
    captures(s, &format!(r#"data-id="{}"[^>]*><path d="([^"]+)""#, regex::escape(id)))[1].clone()
}
fn assert_no_edge_ink(s: &str) {
    assert!(!has(s, r#"pele-casings|stroke="var\(--pele-fg-on"#));
}
fn three_edges(s: &str) -> Vec<Vec<String>> {
    matches(s, r#"class="pele-edge [^"]*" data-id="[^"]+"([^>]*)><path d="([^"]+)""#)
}
fn three_edge_assertions(s: &str) {
    let c = casing(s);
    let es = three_edges(s);
    assert_eq!(es.len(), 3);
    assert_eq!(es.iter().map(|e| c.contains(&format!("<path d=\"{}\"/>", e[2]))).collect::<Vec<_>>(), [false, true, false]);
    assert_eq!(es.iter().map(|e| e[1].as_str()).collect::<Vec<_>>(), [" stroke=\"var(--pele-fg-on-light,#1f1f1f)\"", "", ""]);
}

#[cfg(feature = "agentflow")]
#[test]
fn agentflow_crossed_direction() {
    let flows = "agentflow-beta LR\nflow S\n direction TB\n s1\n s2\nend\nflow P\n direction TB\n p1\n p2\nend\n";
    let links = "s1 --> p2\ns2 --> p1\n";
    assert_eq!(svg(&format!("{flows}{links}")), svg(&format!("{}{links}", flows.replace(" direction TB\n", ""))));
}
#[cfg(feature = "agentflow")]
#[test]
fn agentflow_contrast() {
    let s=svg("agentflow-beta TB\n  flow f[\"Flow\"]\n    a[\"A\"] --> b[\"B\"]\n  end\n  flow p[\"Packed\"]\n    c[\"C\"]\n  end\n  p@{ view: \"collapsed\" }\n  b --> d[\"D\"]\n  style f fill:#FFF9C4\n  style a fill:#1f3a5f\n  style b fill:#1f3a5f,color:#eee\n  style p fill:navy");
    assert!(has(&s, r#"data-id="f">.*?<text class="pele-cluster-label" fill="var\(--pele-fg-on-light,#1f1f1f\)""#));
    for id in ["a", "p"] {
        assert!(node(&s, id).1.contains(&format!("<text class=\"pele-label\" fill=\"{ON_DARK}\"")));
    }
    for id in ["b", "d"] {
        assert!(!node(&s, id).1.contains("--pele-fg-on"));
    }
}
#[cfg(feature = "agentflow")]
#[test]
fn agentflow_filled_edges() {
    let src="agentflow-beta TB\n  flow f[\"Flow\"]\n    a[\"A\"] --> b[\"B\"]\n  end\n  b --> c[\"C\"]\n  c --> d[\"D\"]\n  style f fill:#FFF9C4";
    three_edge_assertions(&svg(src));
    assert_no_edge_ink(&svg(&src.replace("\n  style f fill:#FFF9C4", "")));
    let held = svg("agentflow-beta TB\n  flow f[Flow]\n    a[A]\n  end\n  f --> a\n  style f fill:white");
    assert!(has(&held, r#"data-id="L_f_a_0" stroke="var\(--pele-fg-on-light,#1f1f1f\)""#));
    assert!(!held.contains("pele-casings"));
}
#[cfg(feature = "c4")]
#[test]
fn c4_contrast() {
    let s=svg("C4Context\nBoundary(b, \"B\", \"Kind\") {\n  System(c, \"C\", \"about c\")\n}\nSystem(d, \"D\", \"about d\")\nSystem(e, \"E\")\nUpdateElementStyle(b, $bgColor=\"lightyellow\")\nUpdateElementStyle(c, $bgColor=\"navy\")\nUpdateElementStyle(d, $bgColor=\"navy\", $fontColor=\"#ddd\")");
    assert_eq!(group(&s, "c").matches(&format!("fill=\"{ON_DARK}\"")).count(), 3);
    let b = group(&s, "b");
    assert!(b.contains(&format!("<text class=\"pele-cluster-label\" font-weight=\"var(--_hw)\" fill=\"{ON_LIGHT}\"")));
    assert!(has(&b, r#"class="pele-c4-type"[^>]* fill="var\(--pele-fg-on-light,#1f1f1f\)""#));
    for id in ["d", "e"] {
        assert!(!group(&s, id).contains("--pele-fg-on"));
    }
}
#[cfg(feature = "c4")]
#[test]
fn c4_filled_edges() {
    let src="C4Context\nBoundary(b, \"B\") {\n  System(x, \"X\")\n  System(y, \"Y\")\n}\nSystem(z, \"Z\")\nSystem(w, \"W\")\nRel(x, y, \"in\")\nRel(y, z, \"out\")\nRel(z, w, \"away\")\nUpdateElementStyle(b, $bgColor=\"lightyellow\")";
    let s = svg(src);
    let c = casing(&s);
    for (id, expected) in [("x-y", false), ("y-z", true), ("z-w", false)] {
        assert_eq!(c.contains(&format!("<path d=\"{}\"/>", edge(&s, id))), expected);
    }
    assert!(has(
        &s,
        r#"data-id="x-y" stroke="var\(--pele-fg-on-light,#1f1f1f\)"><path [^>]*/><path class="pele-marker"[^>]* fill="var\(--pele-fg-on-light,#1f1f1f\)""#
    ));
    assert_no_edge_ink(&svg(&src.replace("UpdateElementStyle(b, $bgColor=\"lightyellow\")", "")));
    assert!(has(
        &svg("C4Context\nBoundary(S, \"S\") {\n  System(A, \"A\")\n}\nRel(A, A, \"self\")\nUpdateElementStyle(S, $bgColor=\"white\")"),
        r#"data-id="A-A" stroke="var\(--pele-fg-on-light,#1f1f1f\)""#
    ));
}
#[cfg(feature = "class")]
#[test]
fn class_contrast() {
    let s=svg("classDiagram\nclass A {\n  <<interface>>\n  +id int\n  +run()\n}\nclass B {\n  +x\n}\nclass C\nclassDef dark fill:#1f3a5f\nclass A:::dark\nstyle B fill:lightyellow,color:#900");
    let a = nested_group(&s, "A");
    assert!(has(&a, r#"<text class="pele-annotation"[^>]* fill="var\(--pele-fg-on-dark,#fff\)">«interface»<"#));
    assert!(a.contains(&format!("<text class=\"pele-label\" font-weight=\"var(--_hw)\" fill=\"{ON_DARK}\"")));
    assert!(has(&a, r#"<g class="pele-members" font-size="\d+" fill="var\(--pele-fg-on-dark,#fff\)">"#));
    assert_eq!(a.matches(&format!(" fill=\"{ON_DARK}\"")).count(), 3);
    let b = nested_group(&s, "B");
    assert!(!b.contains("--pele-fg-on"));
    assert!(b.contains("<text class=\"pele-label\" font-weight=\"var(--_hw)\" style=\"fill:#900;\""));
    assert!(!nested_group(&s, "C").contains("--pele-fg-on"));
}
#[cfg(feature = "er")]
#[test]
fn er_contrast() {
    let s=svg("erDiagram\n A { int id PK \"key\" }\n B { int id }\n C { int id }\n D { int id }\n subgraph G\n  E\n end\n style A fill:navy\n style B fill:#BBDEFB\n style C fill:#BBDEFB,color:#333\n style G fill:#FFF9C4");
    let a = nested_group(&s, "A");
    assert!(a.contains(&format!("<g class=\"pele-er-stripes\" fill=\"{ON_LIGHT}\" opacity=\"0.6\">")));
    for cell in ["pele-label", "pele-er-type", "pele-er-name", "pele-er-keys", "pele-er-comment"] {
        assert!(has(&a, &format!(r#"<text class="{cell}"[^>]* fill="var\(--pele-fg-on-dark,#fff\)""#)));
    }
    let b = nested_group(&s, "B");
    assert!(b.contains(&format!("<text class=\"pele-label\" fill=\"{ON_LIGHT}\"")));
    assert!(b.contains(&format!("<g class=\"pele-er-stripes\" fill=\"{ON_DARK}\" opacity=\"0.6\">")));
    let c = nested_group(&s, "C");
    assert!(!c.contains("--pele-fg-on-light"));
    assert!(c.contains("<text class=\"pele-er-type\" fill=\"var(--_m)\" style=\"fill:#333;\""));
    let d = nested_group(&s, "D");
    assert!(d.contains("<g class=\"pele-er-stripes\" fill=\"var(--_bg)\">"));
    assert!(!d.contains("--pele-fg-on"));
    assert!(has(&s, r#"data-id="G">.*?<text class="pele-cluster-label" fill="var\(--pele-fg-on-light,#1f1f1f\)""#));
}
#[cfg(feature = "er")]
#[test]
fn er_opaque_composite() {
    let s = svg("erDiagram\n subgraph g\n  A\n end\n subgraph h\n  B\n end\n style g fill:#fee");
    assert!(has(&s, r#"data-id="g"><rect [^>]*fill="var\(--_a\)" stroke="#));
    assert!(has(&s, r#"data-id="h"><rect [^>]*fill-opacity="0.5""#));
}
#[cfg(feature = "er")]
#[test]
fn er_filled_edges() {
    let src =
        "erDiagram\n subgraph g\n  A ||--o{ B : has\n end\n B ||--|| C : is\n C ||--|| D : of\n C ||--o{ C : self\n style g fill:#BBDEFB";
    let s = svg(src);
    let c = casing(&s);
    let es = matches(&s, r#"class="pele-edge [^"]*" data-id="[^"]+"[^>]*><path d="([^"]+)""#);
    assert_eq!(es.iter().map(|e| c.contains(&format!("<path d=\"{}\"/>", e[1]))).collect::<Vec<_>>(), [false, true, false, false]);
    assert!(has(&s, r#"class="pele-edge [^"]*" data-id="[^"]+" stroke="var\(--pele-fg-on-light,#1f1f1f\)"><path d="[^"]+""#));
    assert_eq!(matches(&s, r#"stroke="var\(--pele-fg-on"#).len(), 1);
    assert!(has(
        &svg("erDiagram\n subgraph S\n  A ||--o{ B : has\n end\n style S fill:black"),
        r#"<circle class="pele-marker"[^>]* fill="black"/>"#
    ));
    assert!(!c.contains("pele-marker"));
    assert_no_edge_ink(&svg(&src.replace("\n style g fill:#BBDEFB", "")));
}
#[cfg(feature = "requirement")]
#[test]
fn requirement_contrast() {
    let s=svg("requirementDiagram\nrequirement a {\nid: 1\ntext: words\n}\nelement b {\ntype: t\n}\nelement c {\n}\nstyle a fill:#1f3a5f\nstyle b fill:#1f3a5f,color:#eee\n");
    for part in ["pele-stereotype", "pele-label", "pele-field", "pele-value"] {
        assert!(has(&nested_group(&s, "a"), &format!(r#"<text class="{part}"[^>]* fill="var\(--pele-fg-on-dark,#fff\)""#)));
    }
    let b = nested_group(&s, "b");
    assert!(has(&b, r#"<text class="pele-label"[^>]* style="fill:#eee;""#));
    assert!(!b.contains("--pele-fg-on"));
    assert!(has(&nested_group(&s, "c"), r#"<text class="pele-stereotype"[^>]* fill="var\(--_m\)""#));
}
#[cfg(feature = "state")]
#[test]
fn state_opaque_composite() {
    let s = svg("stateDiagram-v2\n  state Busy {\n    a\n  }\n  state Idle {\n    b\n  }\n  classDef warm fill:#FFF9C4\n  class Busy warm");
    let rs = matches(&s, r#"<rect [^>]*fill="var\(--_a\)"[^>]*>"#);
    assert_eq!(rs.iter().filter(|r| r[0].contains("fill:#FFF9C4")).count(), 1);
    assert!(!rs.iter().find(|r| r[0].contains("fill:#FFF9C4")).unwrap()[0].contains("fill-opacity"));
    assert_eq!(rs.iter().filter(|r| r[0].contains("fill-opacity=\"0.5\"")).count(), 1);
}
#[cfg(feature = "state")]
#[test]
fn state_contrast() {
    let s=svg("stateDiagram-v2\ndirection TB\n classDef notMoving fill:white\n classDef badBadEvent fill:#f00,color:white,font-weight:bold\n [*] --> Still\n Still --> Moving\n Moving --> Crash\n class Still notMoving\n class Crash badBadEvent\n state \"Described\" as d : first line\n style d fill:#14201a\n state Busy {\n b\n }\n style Busy fill:#FFF9C4");
    for p in [
        r#"data-id="Still"[^>]*><rect[^>]*/><text class="pele-label" fill="var\(--pele-fg-on-light,#1f1f1f\)""#,
        r#"data-id="Crash"[^>]*><rect[^>]*/><text class="pele-label" style="fill:white;font-weight:bold;""#,
        r#"data-id="Moving"[^>]*><rect[^>]*/><text class="pele-label" x="#,
        r#"data-id="d"[^>]*><rect[^>]*/><path[^>]*/><text class="pele-label" fill="var\(--pele-fg-on-dark,#fff\)""#,
        r#"<text class="pele-state-description" font-size="\d+" fill="var\(--pele-fg-on-dark,#fff\)"[^>]*>first line<"#,
        r#"data-id="Busy">.*?<text class="pele-cluster-label" fill="var\(--pele-fg-on-light,#1f1f1f\)"[^>]*>Busy<"#,
    ] {
        assert!(has(&s, p), "{p}");
    }
}
#[cfg(feature = "state")]
#[test]
fn state_filled_edges() {
    let src = "state Busy {\n  a --> b\n}\nstate Idle {\n  c --> d\n}\nb --> e\ne --> c\nstyle Busy fill:#FFF9C4";
    let s = svg(&format!("stateDiagram-v2\n{src}"));
    let c = casing(&s);
    let g = |id: &str| captures(&s, &format!(r#"<g class="pele-edge[^"]*"[^>]* data-id="{id}">.*?</g>"#))[0].clone();
    let path = |id: &str| captures(&g(id), r#"<path d="([^"]+)""#)[1].clone();
    assert!(g("edge0").contains(&format!("class=\"pele-edge pele-transition\" stroke=\"{ON_LIGHT}\"")));
    assert!(g("edge0").contains(&format!("fill=\"{ON_LIGHT}\"")));
    assert!(!c.contains(&path("edge0")));
    assert!(c.contains(&format!("<path d=\"{}\"/>", path("edge2"))));
    assert!(!g("edge2").contains("--pele-fg-on"));
    assert!(!c.contains(&path("edge3")));
    assert_no_edge_ink(&svg(&format!("stateDiagram-v2\n{}", src.replace("\nstyle Busy fill:#FFF9C4", ""))));
}
#[cfg(feature = "swimlane")]
#[test]
fn swimlane_contrast() {
    let s=svg("swimlane-beta LR\n  subgraph L[Light]\n    a\n  end\n  subgraph D[Dark]\n    b\n  end\n  subgraph P[Plain]\n    c\n  end\n  style L fill:#BBDEFB\n  style D fill:#1f3a5f");
    assert!(s.contains(&format!("<text class=\"pele-cluster-label\" fill=\"{ON_LIGHT}\"")));
    for p in [
        r#"data-id="D">.*?<text class="pele-cluster-label" fill="var\(--pele-fg-on-dark,#fff\)""#,
        r#"data-id="P">.*?<text class="pele-cluster-label" fill="var\(--_m\)""#,
        r#"data-id="a"[^>]*><rect [^>]*/><text class="pele-label" x="#,
    ] {
        assert!(has(&s, p), "{p}");
    }
}
#[cfg(feature = "swimlane")]
#[test]
fn swimlane_band_edges() {
    let src="swimlane-beta TB\nsubgraph S[\"one<br/>two<br/>three<br/>four<br/>five<br/>six<br/>seven<br/>eight\"]\n  A\nend\nS --> S\nstyle S fill:black";
    assert!(has(&svg(src), r#"class="pele-edge[^"]*" data-id="[^"]+" stroke="var\(--pele-fg-on-dark,#fff\)""#));
    assert!(!has(&svg(&src.replace("\nstyle S fill:black", "")), r"pele-casings|--pele-fg-on"));
}
#[cfg(feature = "usecase")]
#[test]
fn usecase_opaque_boundary() {
    let s = svg("usecase-beta\nsystemBoundary S\n  U\nend\nsystemBoundary T\n  V\nend\nstyle S fill:#fee");
    let body = |id: &str| {
        let tail = &s[s.find(&format!("data-id=\"{id}\">")).unwrap()..];
        matches(tail, r"<rect [^>]*>").into_iter().find(|r| r[0].contains("fill=\"var(--_a)\"")).unwrap()[0].clone()
    };
    assert!(!body("S").contains("fill-opacity"));
    assert!(body("T").contains("fill-opacity=\"0.5\""));
}
#[cfg(feature = "usecase")]
#[test]
fn usecase_contrast() {
    let s=svg("usecase-beta\nactor A <<Human>>\nU(Use) <<Core>>\nW(Own)\nX(Plain)\njson J@{ \"k\": \"v\" }\nsystemBoundary S\n  V\nend\nstyle U fill:#1f3a5f\nstyle A fill:#1f3a5f\nstyle W fill:#1f3a5f,color:#eee\nstyle J fill:navy\nstyle S fill:#FFF9C4");
    let n = |id: &str| captures(&s, &format!(r#"data-id="{id}"[^>]*>(.*?)</g>(?:<g|</g>)"#))[1].clone();
    assert!(has(&n("U"), r#"<text class="pele-stereotype"[^>]* fill="var\(--pele-fg-on-dark,#fff\)""#));
    assert!(has(&n("U"), r#"<text class="pele-label" fill="var\(--pele-fg-on-dark,#fff\)"[^>]*>Use<"#));
    for p in [
        r#"data-id="A"[^>]*>.*?<text class="pele-stereotype"[^>]* fill="var\(--_m\)""#,
        r#"data-id="A"[^>]*>.*?<text class="pele-label" x=[^>]*>A<"#,
    ] {
        assert!(has(&s, p), "{p}");
    }
    assert!(!n("W").contains("--pele-fg-on"));
    for part in ["key", "value"] {
        assert!(has(&n("J"), &format!(r#"<text class="pele-json-{part}" fill="var\(--pele-fg-on-dark,#fff\)""#)));
    }
    assert!(has(&s, r#"data-id="S">.*?<text class="pele-cluster-label" fill="var\(--pele-fg-on-light,#1f1f1f\)""#));
    assert!(!n("X").contains("--pele-fg-on"));
}
#[cfg(feature = "usecase")]
#[test]
fn usecase_filled_edges() {
    let src = "actor User\nsystemBoundary S\n  Pay\n  Ship\nend\nPay --> Ship\nUser --> Pay\nUser --> Other\nstyle S fill:#BBDEFB";
    three_edge_assertions(&svg(&format!("usecase-beta\n{src}")));
    assert_no_edge_ink(&svg(&format!("usecase-beta\n{}", src.replace("\nstyle S fill:#BBDEFB", ""))));
    assert!(has(
        &svg("usecase-beta\nsystemBoundary S\n  Pay\n  Ship\nend\nPay --|> Ship\nstyle S fill:black"),
        r#"class="pele-marker"[^>]* fill="black" stroke="var\(--pele-fg-on-dark,#fff\)""#
    ));
}
#[cfg(feature = "venn")]
#[test]
fn venn_overlap_contrast() {
    let s=svg("venn-beta\nset A\nset B\nset C\nunion A,B[\"ab\"]\n  text T[\"note\"]\nunion B,C[\"bc\"]\nstyle A,B fill:#1f3a5f\nstyle A fill:navy\nstyle B,C fill:#1f3a5f,color:#eee");
    for p in [
        r#"<text class="pele-label" font-size="\d+" fill="var\(--pele-fg-on-dark,#fff\)"[^>]*>ab<"#,
        r#"<text class="pele-venn-text" data-id="T" font-size="\d+" fill="var\(--pele-fg-on-dark,#fff\)"[^>]*>note<"#,
        r#"<text class="pele-label" font-size="\d+" style="fill:#eee;"[^>]*>bc<"#,
        r#"<text class="pele-label" x=[^>]*>A<"#,
    ] {
        assert!(has(&s, p), "{p}");
    }
}
#[cfg(feature = "venn")]
#[test]
fn venn_last_fill() {
    let covered=svg("venn-beta\nset A: 100\nset B: 100\nset C: 100\nunion A,B,C[\"triple\"]: 50\nunion A,B[\"ab\"]: 70\nunion A,C[\"ac\"]: 70\nunion B,C[\"bc\"]: 70\nstyle A,B,C fill:black\nstyle A,B fill:white");
    assert!(has(&covered, r#"fill="var\(--pele-fg-on-light,#1f1f1f\)"[^>]*>triple<"#));
    assert!(has(&svg("venn-beta\nset A\nstyle A fill:black,fill-opacity:1"), r#"fill="var\(--pele-fg-on-dark,#fff\)"[^>]*>A<"#));
    assert!(has(
        &svg("venn-beta\nset A: 100\nset B: 100\nunion A,B[\"x\"]: 5\nstyle A,B fill:black"),
        r#"fill="var\(--pele-fg-on-dark,#fff\)"[^>]*>x<"#
    ));
}
