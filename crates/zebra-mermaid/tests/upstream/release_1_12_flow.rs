use super::*;
fn edge_group(s: &str, id: &str) -> String {
    captures(s, &format!(r#"<g class="pele-edge[^"]*" data-id="{}"[^>]*>.*?</g>"#, regex::escape(id)))[0].clone()
}
fn edge_path(s: &str, id: &str) -> String {
    captures(s, &format!(r#"data-id="{}"[^>]*><path d="([^"]+)""#, regex::escape(id)))[1].clone()
}
#[test]
fn opaque_subgraph() {
    let s = svg("flowchart LR\n  subgraph S\n    A\n  end\n  subgraph T\n    B\n  end\n  style S fill:#BBDEFB");
    assert!(has(&s, r#"data-id="S"><rect [^>]*fill="var\(--_a\)" stroke="var\(--_b\)" style="[^"]*fill:#BBDEFB;"#));
    assert!(has(&s, r#"data-id="T"><rect [^>]*fill="var\(--_a\)" fill-opacity="0.5""#));
}
#[test]
fn contrasting_labels() {
    let s=svg("flowchart LR\n  subgraph S[\"Cluster\"]\n    A[Start] -- Launch --> B[Worker]\n  end\n  C[Output]\n  D[Dark]\n  E[Own]\n  F[Plain]\n  G[Words]\n  B --> C\n  style S fill:#BBDEFB\n  style C fill:#BBDEFB\n  classDef dark fill:navy\n  class D dark\n  style E fill:#BBDEFB,color:#333\n  style G fill:#000\n  G@{ shape: text }");
    for p in [
        r#"data-id="S">.*?<text class="pele-cluster-label" fill="var\(--pele-fg-on-light,#1f1f1f\)"[^>]*>Cluster<"#,
        r#"data-id="C"[^>]*>.*?<text class="pele-label" fill="var\(--pele-fg-on-light,#1f1f1f\)"[^>]*>Output<"#,
        r#"data-id="D"[^>]*>.*?<text class="pele-label" fill="var\(--pele-fg-on-dark,#fff\)"[^>]*>Dark<"#,
        r#"data-id="E"[^>]*>.*?<text class="pele-label" style="fill:#333;"[^>]*>Own<"#,
        r#"data-id="F"[^>]*>.*?<text class="pele-label" x=[^>]*>Plain<"#,
        r#"data-id="G"[^>]*>.*?<text class="pele-label" x=[^>]*>Words<"#,
        r#"data-id="A"[^>]*>.*?<text class="pele-label" x=[^>]*>Start<"#,
        r#"class="pele-edge-label"[^>]*><rect [^>]*/><text x=[^>]*>Launch<"#,
    ] {
        assert!(has(&s, p), "{p}");
    }
}
#[test]
fn edges_on_and_across_fills() {
    let source="flowchart LR\n  subgraph S[\"Cluster\"]\n    A[Start] -- Launch --> B[Worker]\n    subgraph N\n      G --> H\n    end\n  end\n  subgraph T\n    D --> E\n  end\n  C[Output]\n  B --> C\n  C ==> D\n  B --> B\n  F --> C\n  B --> G\n  A --> H\n  style S fill:#BBDEFB\n  style N fill:navy\n  style C fill:#BBDEFB";
    let s = svg(source);
    let c =
        captures(&s, r#"<g class="pele-casings" fill="none" stroke="var\(--_bg\)" stroke-width="5"[^>]* pointer-events="none">(.*?)</g>"#)
            [1]
        .clone();
    let cased = |id| c.contains(&format!("<path d=\"{}\"/>", edge_path(&s, id)));
    for id in ["L_A_B_0", "L_B_B_0"] {
        assert!(!cased(id));
        let g = edge_group(&s, id);
        assert!(g.contains(&format!(" stroke=\"{ON_LIGHT}\"")));
        assert!(g.contains("class=\"pele-marker\""));
        assert!(has(&g, &format!(r#"class="pele-marker"[^>]* fill="{}""#, regex::escape(ON_LIGHT))));
    }
    assert!(edge_group(&s, "L_G_H_0").contains(&format!(" stroke=\"{ON_DARK}\"")));
    for id in ["L_B_C_0", "L_B_G_0", "L_A_H_0"] {
        assert!(cased(id));
        assert!(!edge_group(&s, id).contains("--pele-fg-on"));
    }
    for id in ["L_C_D_0", "L_D_E_0", "L_F_C_0"] {
        assert!(!cased(id));
        assert!(!edge_group(&s, id).contains("--pele-fg-on"));
    }
    assert!(has(&c, r#"Z" fill="var\(--_bg\)" stroke="var\(--_bg\)"/>"#));
    assert!(!c.contains("pele-marker"));
    assert!(s.find("class=\"pele-casings\"").unwrap() < s.find("class=\"pele-edges\"").unwrap());
    let plain = svg(&re(r"\n  style [SN] fill:\S+").replace_all(source, ""));
    assert!(!has(&plain, r#"pele-casings|--pele-fg-on-[a-z]+,#[0-9a-f]+\)" stroke|class="pele-edge[^>]*--pele-fg-on"#));
}
#[test]
fn marker_reaches_nested_fill() {
    use zebra_mermaid::svg::edges::{over_fills, Fill};
    let fills = [
        Fill { bounds: [-20., -20., 100., 100.], shape: " style=\"fill:white;\"".into() },
        Fill { bounds: [10., 0., 50., 50.], shape: " style=\"fill:navy;\"".into() },
    ];
    assert_eq!(over_fills("M0,10L8,10", &[15., 10.], &fills, "", None), Some(""));
    assert_eq!(over_fills("M0,10L8,10", &[9., 10.], &fills, "", None), Some(ON_LIGHT));
    assert_eq!(over_fills("M-20,10L8,10", &[9., 10.], &fills, "", None), Some(ON_LIGHT));
}
#[test]
fn curve_enters_fill() {
    let s = svg("flowchart LR\nsubgraph S\n  A\n  B\nend\nD\nE --> A\nD --> F\nF --> C\nB --> C\nstyle S fill:white");
    let c = &captures(&s, r#"<g class="pele-casings"[^>]*>(.*)</g><g class="pele-edges""#)[1];
    let edge = |id: &str| captures(&s, &format!(r#"data-id="{id}"><path d="([^"]+)""#))[1].clone();
    assert!(c.contains(&format!("<path d=\"{}\"/>", edge("L_F_C_0"))));
    assert!(!c.contains(&format!("<path d=\"{}\"/>", edge("L_D_F_0"))));
}
#[test]
fn casing_width_and_opacity() {
    let c = |link: &str, fill: &str| {
        let s = svg(&format!("flowchart TB\nsubgraph S\n  A\nend\nA --x B\nstyle S fill:{fill}\n{link}"));
        re(r#"<g class="pele-casings"[^>]*>(.*)</g><g class="pele-edges""#).captures(&s).map_or(String::new(), |m| m[1].to_owned())
    };
    assert!(has(&c("linkStyle 0 stroke-width:10px", "white"), r#"^<g stroke-width="14"><path [^>]*/><path d="[^"]+"/></g>$"#));
    assert!(has(&c("linkStyle 0 stroke-width:1em", "white"), r#"^<g style="stroke-width:calc\(1em \+ 4px\);">"#));
    assert!(has(&c("linkStyle 0 opacity:0,stroke-opacity:0.5", "white"), r#"^<g style="opacity:0;stroke-opacity:0.5;">"#));
    assert_eq!(c("linkStyle 0 stroke:none", "white"), "");
    for fill in ["none", "transparent", "#0000"] {
        assert_eq!(c("", fill), "");
    }
    assert!(!has(&svg("flowchart TB\nsubgraph S\n  A --> B\nend\nstyle S fill:black,opacity:0"), r"pele-casings|--pele-fg-on"));
}
#[test]
fn crossed_group_direction() {
    let groups =
        "flowchart LR\n  subgraph S\n    direction TB\n    s1\n    s2\n  end\n  subgraph P\n    direction TB\n    p1\n    p2\n  end\n";
    let edges = "  s1 --> p2\n  s2 --> p1\n";
    assert_eq!(svg(&format!("{groups}{edges}")), svg(&format!("{}{edges}", re(r" *direction TB\n").replace_all(groups, ""))));
    let s=svg("flowchart LR\n  subgraph one\n    direction TB\n    a1 --> a2\n  end\n  subgraph two\n    direction TB\n    b1 --> b2\n  end\n  subgraph outer\n    direction TB\n    subgraph inner\n      direction TB\n      c1 --> c2\n    end\n    c3\n  end\n  x --> one\n  x --> b1\n  c3 --> x");
    assert_eq!(at(&s, "a1")[0], at(&s, "a2")[0]);
    assert_eq!(at(&s, "b1")[1], at(&s, "b2")[1]);
    assert_eq!(at(&s, "c1")[0], at(&s, "c2")[0]);
}
