//! Independent 1:1 ports of the Pele 1.10.0 release regressions (MIT, kepano 2026).
//! The exact upstream case and language/API correspondence are recorded in ports.json.
use zebra_mermaid::{diagrams::DiagramModel, parse, render, ErrorCode, ParseOptions, RenderOptions};

fn model(source: &str) -> zebra_mermaid::ParsedDiagram {
    parse(source, &ParseOptions::default()).unwrap()
}

fn svg(source: &str) -> String {
    render(source, &RenderOptions::default()).unwrap().svg
}

#[cfg(feature = "c4")]
#[test]
fn c4_bare_title_and_tab() {
    let p = model("C4Context\ntitle\nPerson(a, \"Alice\")\nPerson(b, \"Bob\")");
    let DiagramModel::C4(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.title, None);
    assert_eq!(m.shapes.iter().map(|s| s.element.label.as_str()).collect::<Vec<_>>(), ["Alice", "Bob"]);
    let p = model("C4Context\ntitle\tOwn\nSystem(a, \"A\")");
    let DiagramModel::C4(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.title.as_deref(), Some("Own"));
}

#[cfg(feature = "eventmodeling")]
#[test]
fn eventmodeling_wraps_emoji() {
    let s = svg(&format!("eventmodeling\ntf 01 cmd A {{{}😀Z}}", "a".repeat(47)));
    assert!(s.contains(&format!(">{}😀</tspan>", "a".repeat(47))));
    assert!(s.contains(">Z</tspan>"));
}

#[cfg(feature = "flowchart")]
#[test]
fn flowchart_written_entity_errors() {
    let e = render("flowchart TD\nA[a#35;b] --> -->", &RenderOptions::default()).unwrap_err();
    assert!(e.message.starts_with("Parse error on line 2:\nA[a#35;b] --> -->\n"));
    let e = render("flowchart TD\na@{shape: \"#35;x\"}", &RenderOptions::default()).unwrap_err();
    assert_eq!(e.code, ErrorCode::Semantic);
    assert_eq!(e.message, "No such shape: #35;x.");
}

#[cfg(feature = "flowchart")]
#[test]
fn flowchart_written_positions() {
    let e = render("flowchart TD\nA[a#35;b] --> -->", &RenderOptions::default()).unwrap_err();
    assert_eq!((e.line, e.column), (2, 15));
    assert_eq!(e.snippet, "A[a#35;b] --> -->\n--------------^");
    let e = render("---\ntitle: T\n---\n\n%% A comment\nflowchart TD\n  A --> B\n  C --> --> D", &RenderOptions::default()).unwrap_err();
    assert_eq!((e.line, e.column), (8, 9));
    assert!(e.message.starts_with("Parse error on line 8:\n  C --> --> D\n--------^\n"));
    let e = render("flowchart TD\n  A{a}  \n\n  C --> --> D", &RenderOptions::default()).unwrap_err();
    assert_eq!((e.line, e.column), (4, 9));
}

#[cfg(feature = "gantt")]
#[test]
fn gantt_bare_title_and_tab() {
    let source = "gantt\ntitle\nsection Work\nTask :2026-01-01, 1d";
    let p = model(source);
    let DiagramModel::Gantt(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.title, None);
    // Scheduling is performed during render in Rust; the task labels and sections are
    // exposed directly by GanttModel. Use the original NOW in UTC for the scheduling call.
    render(source, &RenderOptions { now: Some(1_389_873_600_000.0), ..Default::default() }).unwrap();
    assert_eq!(m.tasks.iter().map(|t| (t.task.as_str(), t.section.as_str())).collect::<Vec<_>>(), [("Task ", "Work")]);
    let p = model("gantt\ntitle\tPlan\n");
    let DiagramModel::Gantt(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.title.as_deref(), Some("Plan"));
}

#[cfg(feature = "gantt")]
#[test]
fn gantt_comment_after_task_name() {
    let source = "gantt\ndateFormat YYYY-MM-DD\nA%% label : 2026-10-24, 1d";
    let p = model(source);
    let DiagramModel::Gantt(m) = p.model() else { panic!("wrong type") };
    render(source, &RenderOptions { now: Some(1_389_873_600_000.0), ..Default::default() }).unwrap();
    assert_eq!(m.tasks.iter().map(|t| t.task.as_str()).collect::<Vec<_>>(), ["A%% label "]);
}

#[cfg(feature = "journey")]
#[test]
fn journey_bare_title() {
    let p = model("journey\ntitle\nsection Work\nTask: 5: Me");
    assert_eq!(p.mermaid_type().as_str(), "journey");
    let DiagramModel::Journey(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.title, None);
    assert_eq!(m.sections, ["Work"]);
    let p = model("journey\n  title\u{a0}Kept\n  Task: 5: Me");
    let DiagramModel::Journey(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.title.as_deref(), Some("Kept"));
}

#[cfg(feature = "journey")]
#[test]
fn journey_comment_preserves_character() {
    let p = model("journey\nA%% label: 5: Me");
    assert_eq!(p.mermaid_type().as_str(), "journey");
    let DiagramModel::Journey(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.tasks.iter().map(|t| t.task.as_str()).collect::<Vec<_>>(), ["A%% label"]);
    let e = render("journey\n  Task: 5: Me;%% c\n", &RenderOptions::default()).unwrap_err();
    assert!(e.message.contains("got 'INVALID'"));
}

#[cfg(feature = "quadrantChart")]
#[test]
fn quadrant_comment_preserves_character() {
    let p = model("quadrantChart\nquadrant-1 A%% note");
    let DiagramModel::QuadrantChart(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.quadrants[0], Some(zebra_mermaid::diagrams::quadrant::QuadrantText { text: "A".into(), markdown: false }));
    assert!(parse("quadrantChart\nquadrant-1 A:%% note", &ParseOptions::default()).is_err());
}

#[cfg(feature = "radar")]
#[test]
fn radar_scales_largest_doubles() {
    let huge = [format!("1{}", "0".repeat(308)), format!("5{}", "0".repeat(307)), format!("25{}", "0".repeat(306))].join(",");
    let curve = |values: &str| -> Option<String> {
        let s = svg(&format!("radar-beta\n axis A, B, C\n curve c{{{values}}}\n graticule polygon"));
        let polygon = s.split("<polygon class=\"pele-curve\"").nth(1)?.split('>').next()?;
        Some(polygon.split(" points=\"").nth(1)?.split('"').next()?.to_owned())
    };
    assert_eq!(curve(&huge), curve("4,2,1"));
}

#[cfg(feature = "sankey")]
#[test]
fn sankey_numeric_prefixes_and_rejections() {
    assert!(svg("sankey\na,b,50%\nb,c, 2e1 kWh\n").contains("<title>b → c: 20</title>"));
    for value in ["abc", "", "Infinity", "1e400", "-"] {
        let e = render(&format!("sankey\na,b,1\nb,c,{value}\n"), &RenderOptions::default()).unwrap_err();
        assert_eq!(e.code, ErrorCode::Syntax, "{value}");
        assert_eq!((e.line, e.column), (3, 5), "{value}");
    }
}

#[cfg(feature = "sankey")]
#[test]
fn sankey_error_after_blank_lines() {
    let e = parse("sankey\n\na,b,1\n\nb,c,abc", &ParseOptions::default()).unwrap_err();
    assert_eq!((e.line, e.column), (5, 5));
    let e = parse("sankey\n\na,b,1\n\nb,c,abc", &ParseOptions::default()).unwrap_err();
    assert!(e.message.starts_with("Parse error on line 5:\nb,c,abc\n"));
}

#[cfg(feature = "sankey")]
#[test]
fn sankey_non_positive_empty_flows() {
    let s = svg("sankey\na,b,0\na,c,-3\nb,d,0\nc,d,2\n");
    assert!(!s.contains("NaN"));
    assert_eq!(s.matches("class=\"pele-sankey-link\"").count(), 4);
    assert!(s.contains("<title>a → b: 0</title>"));
}

#[cfg(feature = "sequence")]
#[test]
fn sequence_bare_title() {
    for source in ["sequenceDiagram\ntitle\nA->>B: Hi\nB->>A: Bye", "sequenceDiagram\ntitle: \nA->>B: Hi\nB->>A: Bye"] {
        let p = model(source);
        let DiagramModel::Sequence(m) = p.model() else { panic!("wrong type") };
        assert_eq!(m.title, None, "{source}");
        assert_eq!(m.messages.iter().map(|m| m.message.text().unwrap()).collect::<Vec<_>>(), ["Hi", "Bye"], "{source}");
    }
    let p = model("sequenceDiagram\n  title\u{a0}My title\n  A->>B: x");
    let DiagramModel::Sequence(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.title.as_deref(), Some("My title"));
}

#[cfg(feature = "sequence")]
#[test]
fn sequence_comment_preserves_character() {
    let p = model("sequenceDiagram\n  A->>B:%% comment");
    let DiagramModel::Sequence(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.messages.iter().map(|m| m.message.text().unwrap()).collect::<Vec<_>>(), ["%% comment"]);
    assert!(parse("sequenceDiagram\n  autonumber 5%% comment", &ParseOptions::default()).is_err());
}

#[cfg(feature = "timeline")]
#[test]
fn timeline_bare_title() {
    let p = model("timeline\ntitle\n2024 : Event\n2025 : Next");
    assert_eq!(p.mermaid_type().as_str(), "timeline");
    let DiagramModel::Timeline(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.title, None);
    assert_eq!(m.periods.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(), ["2024 ", "2025 "]);
    let p = model("timeline\n  title\u{a0}Kept\n  2024 : Event");
    let DiagramModel::Timeline(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.title.as_deref(), Some("Kept"));
}

#[cfg(feature = "timeline")]
#[test]
fn timeline_comment_preserves_character() {
    let p = model("timeline\nA%% label : event\nB : e2");
    assert_eq!(p.mermaid_type().as_str(), "timeline");
    let DiagramModel::Timeline(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.periods.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(), ["A%% label ", "B "]);
    let e = render("timeline\n  2024 :%% c\n", &RenderOptions::default()).unwrap_err();
    assert!(e.message.contains("got 'INVALID'"));
}

#[cfg(feature = "venn")]
#[test]
fn venn_comment_preserves_character() {
    let p = model("venn-beta\nset A%% note\nset B:5%% note\nunion A,B%% note");
    let DiagramModel::Venn(m) = p.model() else { panic!("wrong type") };
    assert_eq!(
        m.subsets.iter().map(|s| s.sets.iter().map(String::as_str).collect::<Vec<_>>()).collect::<Vec<_>>(),
        [vec!["A"], vec!["B"], vec!["A", "B"]]
    );
    assert_eq!(m.subsets[1].size, 5.0);
}

#[cfg(feature = "venn")]
#[test]
fn venn_bare_title() {
    let p = model("venn-beta\ntitle\nset A\nset title\ntitle Kept\nset B");
    let DiagramModel::Venn(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.title.as_deref(), Some("Kept"));
    assert_eq!(
        m.subsets.iter().map(|s| s.sets.iter().map(String::as_str).collect::<Vec<_>>()).collect::<Vec<_>>(),
        [vec!["A"], vec!["title"], vec!["B"]]
    );
}

#[cfg(feature = "xychart")]
#[test]
fn xychart_clamped_y_bounds() {
    let big = |e| format!("1{}", "0".repeat(e));
    for range in [format!("{} --> {}", big(301), big(302)), format!("-{} --> -{}", big(302), big(301))] {
        assert!(!svg(&format!("xychart\n x-axis [a, b]\n y-axis {range}\n bar [3, 4]")).contains("NaN"), "{range}");
    }
}

#[cfg(feature = "xychart")]
#[test]
fn xychart_comment_preserves_character() {
    let p = model("xychart-beta\nx-axis [a, b]\nbar [1,2]%% note");
    let DiagramModel::Xychart(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.plots.len(), 1);
    assert_eq!(m.plots[0].kind, zebra_mermaid::diagrams::xychart::PlotType::Bar);
    assert_eq!(m.plots[0].title, "");
    assert_eq!(m.plots[0].data, [("a".into(), Some(1.0)), ("b".into(), Some(2.0))]);
    assert!(parse("xychart-beta\ntitle Sales:%% note\nbar [1,2]", &ParseOptions::default()).is_err());
}

#[cfg(feature = "agentflow")]
#[test]
fn agentflow_original_source_positions() {
    let source = "---\nconfig:\n  agentflow:\n    nodeSpacing: 40\n---\nagentflow-beta TB\n  %% a comment that occupies a line\n  alpha[\"Alpha\"]\n  beta[\"Beta\"]\n  alpha --> beta";
    let p = model(source);
    let DiagramModel::Agentflow(m) = p.model() else { panic!("wrong type") };
    let alpha = m.mappings.iter().find(|m| m.id == "alpha").unwrap();
    assert_eq!((alpha.id.as_str(), alpha.kind.as_str()), ("alpha", "vertex"));
    let pos = &alpha.position;
    assert_eq!((pos.start_line, pos.start_column, pos.end_line, pos.end_column, pos.start_index, pos.end_index), (8, 2, 8, 16, 0, 0));
    let edge = m.mappings.iter().find(|m| m.kind == "edge").unwrap();
    assert_eq!((edge.position.start_line, edge.position.end_line), (10, 10));
    assert!(svg(source).contains("data-id=\"alpha\""));
    let e = render("---\ntitle: T\n---\nagentflow-beta TB\n  a@{ shape: rounded }  \n  b -.-> c", &RenderOptions::default()).unwrap_err();
    assert_eq!((e.line, e.column), (6, 8));
    let metadata = "%%{init:\n{}\n}%%\nagentflow-beta\na[\"#35;\"] --> b@{shape: ";
    let e = parse(&format!("{metadata}bad}}"), &ParseOptions::default()).unwrap_err();
    assert_eq!(e.code, ErrorCode::Semantic);
    assert_eq!((e.line, e.column), (5, 16));
    let e = parse(&format!("{metadata}[bad}}"), &ParseOptions::default()).unwrap_err();
    assert_eq!(e.code, ErrorCode::Syntax);
    assert_eq!((e.line, e.column), (5, 16));
    assert!(e.message.ends_with("(5:16)"));
}

#[cfg(feature = "treeView")]
#[test]
fn treeview_box_drawing_original_errors() {
    let e = render("treeView-beta\n│\n├── src/\n│\n│   └── \"a\n", &RenderOptions::default()).unwrap_err();
    assert_eq!(e.line, 5);
    assert!(e.message.contains("line 5"));
    let e = render("treeView-beta\n├── \n", &RenderOptions::default()).unwrap_err();
    assert!(e.message.contains("Line 2: Empty node"));
    let e = render("treeView-beta\n├── a\n   b\n", &RenderOptions::default()).unwrap_err();
    assert!(e.message.contains("Line 3: Unexpected indentation"));
    let e = render("---\ntitle: T\n---\ntreeView-beta\n└─ \"bad", &RenderOptions::default()).unwrap_err();
    assert_eq!((e.line, e.column), (5, 4));
    assert_eq!(e.snippet, "└─ \"bad\n---^");
    assert!(e.message.starts_with("Lexical error on line 5."));
    let e = render("---\ntitle: T\n---\ntreeView-beta\n├── \n", &RenderOptions::default()).unwrap_err();
    assert!(e.message.contains("Line 5: Empty node"));
    let e = render("treeView-beta\n└─ \"ok\"\t\"bad", &RenderOptions::default()).unwrap_err();
    assert_eq!((e.line, e.column), (2, 9));
    let e = render("treeView-beta\n└── \"ok\"\t\t\"bad", &RenderOptions::default()).unwrap_err();
    assert_eq!((e.line, e.column), (2, 11));
    assert_eq!(e.snippet, "└── \"ok\"\t\t\"bad\n----------^");
    let e = render("---\ntitle: T\n---\ntreeView-beta\n\"bad\nhello ─ world\nx\ny", &RenderOptions::default()).unwrap_err();
    assert_eq!((e.line, e.column), (5, 1));
}

#[cfg(feature = "usecase")]
#[test]
fn usecase_original_error_words() {
    let message = |source| {
        let e = render(source, &RenderOptions::default()).unwrap_err();
        format!("{}: {}", e.code, e.message)
    };
    assert_eq!(message("usecase-beta\nactor A actor B"), "syntax: Error parsing usecase diagram: Expecting: one of these possible Token sequences:\n  1. [NEWLINE]\n  2. [EOF]\nbut found: 'actor' at line 2, column 9 [21,26)");
    assert_eq!(message("usecase-beta\nsystemBoundary S\nA --> B\nend"), "syntax: Error parsing usecase diagram: Expecting: one of these possible Token sequences:\n  1. [NEWLINE]\n  2. [EOF]\nbut found: '-->' at line 3, column 3 [32,35)");
    assert_eq!(
        message("usecase-beta\nsystemBoundary S\nA"),
        "syntax: Error parsing usecase diagram: Expecting token of type --> END <-- but found --> '' <-- at line 4, column 1 [32,32)"
    );
    assert_eq!(message("usecase-beta\nA \"b"), "syntax: Error lexing usecase diagram: unexpected character: ->\"<- at offset: 15, skipped 1 characters. at line 2, column 3 [15,16)");
    assert_eq!(message("usecase-beta\nnote for Ghost \"x\""), "semantic: Note target 'Ghost' is unresolved at line 2, column 10 [22,27)");
    let json = message("usecase-beta\njson P@{\"a\":}");
    assert!(json.starts_with("syntax: Invalid JSON: "));
    let column = json.strip_suffix(')').unwrap().rsplit_once(" (line 2, column ").unwrap().1;
    assert!(!column.is_empty() && column.bytes().all(|b| b.is_ascii_digit()));
    let e = render("usecase-beta\njson P@{\"a\":\"#35;\", \"b\":}", &RenderOptions::default()).unwrap_err();
    assert_eq!((e.line, e.column), (2, 25));
    assert!(!e.message.contains(['ﬂ', '°', '¶', 'ß']));
    let e = render("usecase-beta\nactor A\nA --> B\nstyle Z fill:red", &RenderOptions::default()).unwrap_err();
    assert_eq!(e.code, ErrorCode::Semantic);
    assert_eq!(e.diagram_type, Some(zebra_mermaid::DiagramType::Usecase));
    assert_eq!((e.line, e.column), (4, 7));
}

#[cfg(feature = "pie")]
#[test]
fn pie_negative_value_messages() {
    let e = render("pie\n \"#35;x &amp;y\" : -1", &RenderOptions::default()).unwrap_err();
    assert!(e.message.contains("\"#35;x &amp;y\" has invalid value: -1."));
    let e = render("pie\n \"dogs\" : -60.67", &RenderOptions::default()).unwrap_err();
    assert!(e
        .message
        .contains("\"dogs\" has invalid value: -60.67. Negative values are not allowed in pie charts. All slice values must be >= 0."));
}

#[cfg(feature = "radar")]
#[test]
fn radar_named_entries_and_errors() {
    let p = model("radar-beta\n axis A, B, C\n curve c{ C: 3, A: 1, B: 2 }");
    let DiagramModel::Radar(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.curves[0].entries, [1.0, 2.0, 3.0]);
    for (source, message) in [
        ("radar-beta\n axis A[\"Alpha\"], B\n curve c{ B: 2 }", "Missing entry for axis Alpha"),
        ("radar-beta\n axis a[\"#35;x &amp;y\"], b\n curve c{ b: 1 }", "Missing entry for axis #35;x &amp;y"),
        ("radar-beta\n curve c{ B: 2 }", "Axes must be populated before curves for reference entries"),
    ] {
        assert!(render(source, &RenderOptions::default()).unwrap_err().message.contains(message));
    }
}

#[cfg(feature = "venn")]
#[test]
fn venn_rejections_and_written_names() {
    for (source, message) in [
        ("venn-beta\nunion A", "union requires multiple identifiers"),
        ("venn-beta\nset A\nunion A,B", "unknown set identifier: B"),
        ("venn-beta\nset A\nunion A,\"B#35;x\"", "unknown set identifier: B#35;x"),
        ("venn-beta\nset \"B#x\"\nset A\nunion A,\"B#35;x\"", "unknown set identifier: B#35;x"),
    ] {
        assert_eq!(render(source, &RenderOptions::default()).unwrap_err().message, message);
    }
    let e = render("venn-beta\nset A,B", &RenderOptions::default()).unwrap_err();
    assert_eq!(e.message, "Parse error on line 2:\nset A,B\n-----^\nExpecting 'EOF', 'NEWLINE', 'TITLE', 'SET', 'UNION', 'TEXT', 'INDENT_TEXT', 'STYLE', got 'COMMA'");
    let e = render("venn-beta\nset A $", &RenderOptions::default()).unwrap_err();
    assert!(e.message.starts_with("Lexical error on line 2. Unrecognized text."));
}

#[cfg(feature = "sankey")]
#[test]
fn sankey_value_labels_and_entity_affixes() {
    let basic = "sankey\na,b,5\na,c,3\nb,d,4\nc,d,1.239\n";
    let configured = |config: &str| {
        format!("---\nconfig:\n  sankey:\n{}\n---\n{basic}", config.lines().map(|l| format!("    {l}")).collect::<Vec<_>>().join("\n"))
    };
    let shown = svg(basic);
    assert!(shown.contains(">a <tspan class=\"pele-sankey-value\" fill=\"var(--_m)\">8</tspan><"));
    assert!(shown.contains(">5.24</tspan>"));
    let money = svg(&configured("prefix: \"$\"\nsuffix: \" M\""));
    assert!(money.contains(">$8 M</tspan>"));
    assert!(money.contains("<title>a → b: $5 M</title>"));
    let coded = svg(&configured("prefix: \"&euro;\"\nsuffix: \" &amp;lt;\"\nlabelStyle: outlined"));
    assert!(coded.contains(">€8 &amp;lt;</tspan>"));
    assert!(coded.contains("<title>a → b: €5 &amp;lt;</title>"));
    let hidden = svg(&configured("showValues: false"));
    assert!(!hidden.contains("pele-sankey-value"));
    assert!(hidden.contains(">a</text>"));
}

#[cfg(all(feature = "flowchart", feature = "agentflow", feature = "er"))]
#[test]
fn internal_links_refuse_unsafe_hrefs() {
    assert!(svg("flowchart TD\n  A[\"javascript:alert(1)\"]:::internal-link")
        .contains("<a class=\"internal-link\" data-href=\"javascript:alert(1)\">"));
    for kind in ["flowchart TD", "agentflow-beta TB", "erDiagram"] {
        let options = RenderOptions { link_schemes: Some(vec!["about".into()]), ..Default::default() };
        let s = render(&format!("{kind}\n  A[\"javascript:alert(1)\"]:::internal-link"), &options).unwrap().svg;
        assert!(!s.contains(" href="));
    }
}

#[cfg(all(feature = "flowchart", feature = "agentflow"))]
#[test]
fn edge_limit_before_allocation() {
    let ids = |prefix: &str, count: usize| (0..count).map(|i| format!("{prefix}{i}")).collect::<Vec<_>>().join(" & ");
    let product = format!("{} --> {}", ids("a", 2000), ids("b", 2000));
    for source in [format!("flowchart LR\n{product}"), format!("agentflow-beta LR\n{product}")] {
        let started = std::time::Instant::now();
        let e = render(&source, &RenderOptions::default()).unwrap_err();
        let message = e.message.strip_prefix("Edge limit exceeded").unwrap();
        assert!(message.contains("limit is 5000"));
        assert!(started.elapsed().as_secs_f64() * 1000.0 < 2000.0);
    }
    let chain = |count: usize| format!("flowchart LR\n{}", (0..count).map(|i| format!("N{i} --> N{}\n", i + 1)).collect::<String>());
    let big = RenderOptions { limit: Some(usize::MAX), ..Default::default() };
    assert!(render(&chain(5000), &big).unwrap().svg.contains("<svg"));
    assert!(render(&chain(5001), &big).is_err());
    let limited = RenderOptions { max_edges: Some(10), ..Default::default() };
    assert!(render(&chain(11), &limited).unwrap_err().message.contains("Edge limit exceeded. The limit is 10."));
    assert!(parse(&chain(11), &ParseOptions { max_edges: Some(10), ..Default::default() }).unwrap_err().message.contains("limit is 10."));
    if let Err(e) = render(&chain(11), &limited) {
        assert_eq!(e.code, ErrorCode::Limit);
    }
    let raised = format!(
        "---\nconfig:\n  maxEdges: 100000\n  flowchart:\n    maxEdges: 100000\n---\n%%{{init: {{'maxEdges': 100000}}}}%%\n{}",
        chain(11)
    );
    assert!(render(&raised, &limited).unwrap_err().message.contains("limit is 10."));
}
