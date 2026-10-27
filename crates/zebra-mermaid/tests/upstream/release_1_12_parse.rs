use super::*;

#[cfg(feature = "railroad")]
#[test]
fn railroad_title_names_and_late_title() {
    use zebra_mermaid::diagrams::railroad::RailroadNode;
    let p = model("railroad-ebnf-beta\ntitles = a ;");
    let DiagramModel::Railroad(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.title, None);
    assert_eq!(m.rules.len(), 1);
    assert_eq!(m.rules[0].name, "titles");
    assert_eq!(m.node(m.rules[0].definition), &RailroadNode::Nonterminal { name: "a".into() });
    for source in ["railroad-ebnf-beta\nr = a ;\ntitle Late", "railroad-ebnf-beta\nr = title ;"] {
        assert!(parse(source, &ParseOptions::default()).is_err());
    }
}
#[cfg(feature = "architecture")]
#[test]
fn architecture_title_ids() {
    let p=model("architecture-beta\ngroup titles(cloud)[G]\nservice title-db(database)[DB] in titles\nservice title_2 in titles\ntitle-db:R -- L:title_2");
    let DiagramModel::Architecture(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.title, None);
    assert_eq!(m.groups.keys().map(String::as_str).collect::<Vec<_>>(), ["titles"]);
    assert_eq!(m.nodes.keys().map(String::as_str).collect::<Vec<_>>(), ["title-db", "title_2"]);
    assert_eq!(m.edges.iter().map(|e| (e.lhs_id.as_str(), e.rhs_id.as_str())).collect::<Vec<_>>(), [("title-db", "title_2")]);
}
#[cfg(feature = "eventmodeling")]
#[test]
fn event_title_entities() {
    let p = model("eventmodeling\ntf 01 ui titles\ntf 02 cmd title_x\nentity titles\n");
    let DiagramModel::Eventmodeling(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.boxes.iter().map(|b| b.name.as_str()).collect::<Vec<_>>(), ["titles", "title_x"]);
    assert_eq!(m.entities, ["titles"]);
}
#[cfg(feature = "gitGraph")]
#[test]
fn git_title_branches() {
    let p =
        model("gitGraph\n commit\n branch titles\n commit\n branch title/x\n commit\n checkout main\n merge titles\n branch title-2.0\n");
    let DiagramModel::GitGraph(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.lanes, ["main", "titles", "title/x", "title-2.0"]);
    assert_eq!(m.title, None);
    assert!(parse("gitGraph\n commit\n branch title.v2\n checkout title.v2", &ParseOptions::default()).is_ok());
}
#[cfg(feature = "radar")]
#[test]
fn radar_keyword_prefixes() {
    use zebra_mermaid::diagrams::radar::Graticule;
    let p=model("radar-beta\n axis circles, polygonal, trueNorth, falsey, titles\n curve titles{1, 2, 3, 4, 5}, circles{ circles: 1, polygonal 2, trueNorth: 3, falsey 4, titles: 5 }");
    let DiagramModel::Radar(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.axes.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), ["circles", "polygonal", "trueNorth", "falsey", "titles"]);
    assert_eq!(
        m.curves.iter().map(|c| (c.name.as_str(), c.entries.as_slice())).collect::<Vec<_>>(),
        [("titles", &[1., 2., 3., 4., 5.][..]), ("circles", &[1., 2., 3., 4., 5.][..])]
    );
    assert_eq!(m.title, None);
    let p = model("radar-beta\n axis a, b\n showLegend falsecurve x{1, 2}\n graticule polygonaxis c");
    if let DiagramModel::Radar(m) = p.model() {
        assert!(!m.options.show_legend);
        assert_eq!(m.options.graticule, Graticule::Polygon);
        assert_eq!(m.axes.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(), ["a", "b", "c"]);
    }
    assert!(parse("radar-beta\n axis circle, b", &ParseOptions::default()).unwrap_err().message.contains("Parse error"));
}
#[cfg(feature = "railroad")]
#[test]
fn railroad_title_rules_all_notations() {
    for source in [
        "railroad-beta\ntitles = nonterminal(\"title-x\") ;\ntitle-x = terminal(\"a\") ;",
        "railroad-ebnf-beta\ntitles = title-x ;\ntitle-x = \"a\" ;",
        "railroad-abnf-beta\ntitles = title-x ;\ntitle-x = \"a\" ;",
        "railroad-peg-beta\ntitles <- title-x ;\ntitle-x <- \"a\" ;",
    ] {
        let p = model(source);
        let DiagramModel::Railroad(m) = p.model() else { panic!("wrong type") };
        assert_eq!(m.rules.iter().map(|r| r.name.as_str()).collect::<Vec<_>>(), ["titles", "title-x"], "{source}");
        assert!(svg(source).contains("data-id=\"title-x\""), "{source}");
    }
}
#[cfg(feature = "sankey")]
#[test]
fn sankey_row_blanks() {
    let links = |source: &str| {
        let p = model(source);
        let DiagramModel::Sankey(m) = p.model() else { panic!("wrong type") };
        m.links.iter().map(|l| format!("{}>{}:{}", m.nodes[l.source].id, m.nodes[l.target].id, l.value)).collect::<Vec<_>>()
    };
    for (source, expected) in [
        ("sankey-beta \na,b,1", vec!["a>b:1"]),
        ("sankey-beta\n\ta,b,1\nc,d,2", vec!["a>b:1", "c>d:2"]),
        ("sankey-beta\na,b,1\n   \nc,d,2", vec!["a>b:1", "c>d:2"]),
        ("sankey-beta\na,b,1\n \"c d\",e,2", vec!["a>b:1", "c d>e:2"]),
        ("sankey-beta\nd,e,\"2\" \ng,h,3", vec!["d>e:2", "g>h:3"]),
        ("sankey\n\"a  \n  b\",c,1", vec!["a  \n  b>c:1"]),
        ("sankey\na,b,1\n ,c,2", vec!["a>b:1", ">c:2"]),
    ] {
        assert_eq!(links(source), expected);
    }
}
#[cfg(feature = "sankey")]
#[test]
fn sankey_row_error_positions() {
    for (source, line, column) in
        [("sankey \n\t a,b,1 \n  \n \tb,c,abc ", 4, 7), ("sankey\n \"a \n b\", c ,x", 3, 9), ("sankey\n a,b,1 \n c,d", 3, 5)]
    {
        let e = parse(source, &ParseOptions::default()).unwrap_err();
        assert_eq!((e.line, e.column), (line, column));
    }
}
#[cfg(feature = "treeView")]
#[test]
fn tree_title_names() {
    let p = model("treeView-beta\ntitles/\n  title.txt\n  title-page.md\n");
    let DiagramModel::TreeView(m) = p.model() else { panic!("wrong type") };
    let dir = &m.nodes[m.nodes[0].children[0]];
    assert_eq!(
        std::iter::once(dir.name.as_str()).chain(dir.children.iter().map(|&i| m.nodes[i].name.as_str())).collect::<Vec<_>>(),
        ["titles", "title.txt", "title-page.md"]
    );
    let s = svg("treeView-beta\ntitles/\n├── title.txt\n└── title-page.md\n");
    assert!(s.contains(">title.txt</text>"));
    assert!(!s.contains(">.txt</text>"));
    let p = model("treeView-beta\ntitle%% c\naccTitle: A\nroot\n");
    let DiagramModel::TreeView(m) = p.model() else { panic!("wrong type") };
    assert_eq!(m.acc_title.as_deref(), Some("A"));
    assert_eq!(m.nodes[0].children.iter().map(|&i| m.nodes[i].name.as_str()).collect::<Vec<_>>(), ["root"]);
}
#[cfg(feature = "venn")]
#[test]
fn venn_hyphenated_keywords() {
    let p=model("venn-beta\nset set-theory\nset union-x\nunion set-theory,union-x[\"both\"]\n  text-5\ntext set-theory text-books\nstyle set-theory fill:red,x:style-y");
    let DiagramModel::Venn(m) = p.model() else { panic!("wrong type") };
    assert_eq!(
        m.subsets.iter().map(|s| s.sets.iter().map(String::as_str).collect::<Vec<_>>()).collect::<Vec<_>>(),
        [vec!["set-theory"], vec!["union-x"], vec!["set-theory", "union-x"]]
    );
    assert_eq!(m.text_nodes.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), ["-5", "text-books"]);
    assert_eq!(m.styles[0].styles.get("x").map(String::as_str), Some("style-y"));
}
#[cfg(feature = "quadrantChart")]
#[test]
fn quadrant_empty_label_space() {
    let s = svg("quadrantChart\n  P: [0.25, 0.4375]\n  Q: [0.25, 0.4175]");
    let es = elements(&s);
    let p = es.iter().find(|e| e.name == "circle" && e.attr("data-id") == "P").unwrap();
    let text = captures(&s, r"<text[^>]*>P</text>")[0].clone();
    let l = elements(&text);
    assert_eq!(l[0].attr("x"), p.attr("cx"));
    assert!(l[0].n("y") < p.n("cy"));
}
