use super::*;
fn lifelines(s: &str) -> BTreeMap<String, f64> {
    matches(s, r#"<path class="pele-lifeline" data-id="([^"]*)" d="M(-?[\d.]+),"#)
        .into_iter()
        .map(|m| (m[1].clone(), number(&m[2])))
        .collect()
}
#[test]
fn widen_before_wrap() {
    let reported="sequenceDiagram\n  participant U as UE\n  participant M as MeNB (LTE anchor)\n  participant S as SgNB (5G, NR leg)\n  M->>S: SgNB Addition Request (DRB-ID, CG-ConfigInfo)\n  M->>U: RRCConnectionReconfiguration (carries the NR SCG config)\n  U->>M: RRCConnectionReconfigurationComplete";
    let long = "thisIsAnExtremelyLongMessageIdentifierThatShouldHaveRoomToFit";
    let cases = [
        (reported.to_owned(), "i2", 760., 1, json!({})),
        (reported.to_owned(), "i2", 760.5, 1, json!({})),
        (reported.to_owned(), "i2", 760., 1, json!({"monospace":["messages"]})),
        (reported.to_owned(), "i2", 400., 2, json!({})),
        (
            format!(
                "sequenceDiagram\n  participant A\n  participant B\n  participant C\n  B->>A: {}\n  C->>A: {}",
                "x".repeat(168),
                "x".repeat(89)
            ),
            "i1",
            709.,
            1,
            json!({}),
        ),
        (format!("sequenceDiagram\n  A->>B: {long}"), "i0", 300.25, 3, json!({})),
        (
            format!("sequenceDiagram\n  participant A\n  participant B\n  participant C\n  participant D\n  A->>D: {long}"),
            "i0",
            450.,
            2,
            json!({}),
        ),
        (format!("sequenceDiagram\n  A->>+B: {long}\n  Note right of B: note\n  B-->>-A: ok"), "i0", 400., 2, json!({})),
    ];
    for (source, id, max_width, count, mut extra) in cases {
        extra["maxWidth"] = json!(max_width);
        let r = draw_with(&source, extra);
        assert!(r.width <= max_width);
        let label = captures(&r.svg, &format!(r#"<g class="pele-edge-label" data-id="{id}">(.*?)</g>"#))[1].clone();
        assert_eq!(label.matches("<tspan").count().max(1), count);
    }
}
#[test]
fn message_mono() {
    let src = "sequenceDiagram\n  A->>B: a message in the monospace font\n  B->>B: a message to itself\n  Note over A,B: a note";
    let plain = svg(src);
    let mono = svg_with(src, json!({"monospace":["messages"]}));
    assert!(mono.contains("<g class=\"pele-edge-labels\" font-size=\"14\" font-family=\"var(--_fm)\">"));
    assert!(!plain.contains("font-family=\"var(--_fm)\""));
    assert_eq!(mono.matches("font-family=\"var(--_fm)\"").count(), 1);
    let gap = |s: &str| {
        let l = lifelines(s);
        l["B"] - l["A"]
    };
    assert!(gap(&mono) > gap(&plain));
    assert!(draw_with(src, json!({"monospace":["messages"]})).width - gap(&mono) > draw(src).width - gap(&plain));
    for m in [json!([]), json!(["message"]), json!("messages"), json!(["members"])] {
        assert_eq!(svg_with(src, json!({"monospace":m})), plain);
    }
}
#[test]
fn widen_mono() {
    let src="sequenceDiagram\n  participant UE\n  participant gNB\n  participant AMF\n  gNB->>UE: RRCConnectionReconfigurationComplete\n  UE->>gNB: ok";
    let mono = draw_with(src, json!({"maxWidth":760,"monospace":["messages"]}));
    assert!(mono.width <= 760.);
    assert!(mono.width > draw_with(src, json!({"maxWidth":760})).width);
    assert!(mono.svg.contains(">RRCConnectionReconfigurationComplete<"));
}
#[test]
fn contrasting_rect_box_labels() {
    let s=svg("sequenceDiagram\n  box rgb(34, 56, 0) Group\n  participant A\n  participant B\n  end\n  participant C\n  participant D\n  A->>B: boxed\n  C->>D: open\n  rect rgb(191, 223, 255)\n  C->>D: light\n  C->>C: self\n  loop Often\n  D->>C: looped\n  end\n  rect black\n  C->>D: dark\n  end\n  rect rgba(0, 0, 0, .2)\n  C->>D: faint\n  end\n  A->>D: crossing two of the lifelines\n  end");
    let label = |text: &str| captures(&s, &format!(r"<text[^>]*>(?:\[)?{text}(?:\])?</text>"))[0].clone();
    let light = format!("fill=\"{ON_LIGHT}\"");
    let dark = format!("fill=\"{ON_DARK}\"");
    assert!(label("boxed").contains(&dark));
    assert!(s.contains(&format!("<text class=\"pele-cluster-label\" {dark}")));
    assert!(!label("open").contains("fill="));
    for text in ["light", "self", "looped"] {
        assert!(label(text).contains(&light));
    }
    assert!(label("Often").contains(&format!("class=\"pele-frame-label\" {light} stroke=\"none\"")));
    assert!(label("dark").contains(&dark));
    assert!(!label("faint").contains("fill="));
    assert!(has(&s, r#"fill="var\(--_bg\)" stroke="none"/><text x=[^>]*>crossing two of the lifelines<"#));
}
#[test]
fn contrasting_arrows() {
    let s=svg("sequenceDiagram\n  box black Group\n  participant A\n  participant B\n  end\n  participant C\n  A->>B: boxed\n  B->>C: open\n  rect rgb(191, 223, 255)\n  A-->>C: light\n  C-xC: self\n  A-|\\B: solid\n  A-\\\\B: stick\n  end");
    let message = |text: &str| {
        let id = captures(&s, &format!(r#"data-id="(i\d+)">(?:<rect[^>]*/>)?<text[^>]*>{text}<"#))[1].clone();
        captures(&s, &format!(r#"<g class="pele-edge pele-message" data-id="{id}"[^>]*>.*?</g>"#))[0].clone()
    };
    assert!(has(&message("boxed"), r#"^<g [^>]* stroke="var\(--pele-fg-on-dark,#fff\)">"#));
    assert!(message("boxed").contains("class=\"pele-marker\""));
    assert!(has(&message("boxed"), r#"class="pele-marker"[^>]* fill="var\(--pele-fg-on-dark,#fff\)""#));
    assert!(!message("open").contains("--pele-fg-on"));
    assert!(has(&message("light"), r#"^<g [^>]* stroke="var\(--pele-fg-on-light,#1f1f1f\)"><path [^>]*stroke-dasharray"#));
    for text in ["self", "stick"] {
        assert!(has(&message(text), r#"^<g [^>]* stroke="var\(--pele-fg-on-light,#1f1f1f\)">"#));
        assert!(!message(text).contains("var(--_l)"));
    }
    assert!(has(&message("solid"), r#"class="pele-marker" d="[^"]+Z" fill="var\(--pele-fg-on-light,#1f1f1f\)""#));
    assert!(!has(&s, r#"class="pele-lifeline"[^>]*--pele-fg-on"#));
}
#[test]
fn self_message_outside_box() {
    let s = svg("sequenceDiagram\n  autonumber 1000\n  box black\n  participant A\n  end\n  A->>A: x");
    assert!(!has(&s, r#"class="pele-edge pele-message"[^>]* stroke="#));
}
#[test]
fn figures_in_box() {
    let s = svg("sequenceDiagram\n  box black Group\n  actor A\n  participant B\n  end\n  actor C\n  A->>B: hi\n  B->>C: yo");
    assert_eq!(s.matches(&format!("<text class=\"pele-label\" fill=\"{ON_DARK}\"")).count(), 2);
    assert!(has(&s, r#"data-id="A">.*?<text class="pele-label" fill="var\(--pele-fg-on-dark,#fff\)""#));
    for id in ["B", "C"] {
        assert!(has(&s, &format!(r#"data-id="{id}">.*?<text class="pele-label" x="#)));
    }
}
