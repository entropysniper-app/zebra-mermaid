use super::*;
fn balanced_group(s: &str, id: &str) -> String {
    let start = s.find(&format!("data-id=\"{id}\""));
    assert!(start.is_some(), "group {id}");
    let from = s[..start.unwrap()].rfind("<g").unwrap();
    let mut depth = 0;
    for c in re(r"<(/?)g\b").captures_iter(&s[from..]) {
        depth += if &c[1] == "/" { -1 } else { 1 };
        if depth == 0 {
            return s[from..from + c.get(0).unwrap().start()].into();
        }
    }
    s[from..].into()
}
#[cfg(feature = "class")]
#[test]
fn class_members_mono() {
    let src = "classDiagram\nclass A {\n  +title: string\n}";
    let plain = svg(src);
    let mono = svg_with(src, json!({"monospace":["members"]}));
    let width = |s: &str| number(&captures(&balanced_group(s, "A"), r#"<rect[^>]* width="([\d.]+)""#)[1]);
    assert!(mono.contains("<g class=\"pele-members\" font-size=\"14\" font-family=\"var(--_fm)\">"));
    assert!(!plain.contains("font-family=\"var(--_fm)\""));
    assert!(width(&mono) > width(&plain));
}
#[cfg(feature = "class")]
#[test]
fn class_invalid_mono() {
    let src = "classDiagram\nclass A {\n  +title: string\n  +draw()*\n}";
    let plain = svg(src);
    for mono in [json!([]), json!(["member"]), json!(["attributes"]), json!("members"), json!({"members":true}), json!(true), Value::Null] {
        assert_eq!(svg_with(src, json!({"monospace":mono})), plain);
    }
    assert_eq!(svg_with(src, json!({"config":{"class":{"monoMembers":true,"monospace":["members"]}}})), plain);
    assert_eq!(svg_with(src, json!({"monospace":["attributes","members"]})), svg_with(src, json!({"monospace":["members"]})));
}
#[cfg(feature = "class")]
#[test]
fn class_own_font() {
    for rule in ["style A color:black,font-family:serif", "classDef serif color:black,font-family:serif\nclass A:::serif"] {
        let src = format!("classDiagram\nclass A {{\n  +title: string\n}}\nclass B {{\n  +title: string\n}}\n{rule}");
        let s = svg_with(&src, json!({"monospace":["members"]}));
        let members = |s: &str| captures(&balanced_group(s, "A"), r#"<g class="pele-members".*?</g>"#)[0].clone();
        assert_eq!(members(&s), members(&svg(&src)));
        assert!(balanced_group(&s, "B").contains("font-family=\"var(--_fm)\""));
    }
}
#[cfg(feature = "er")]
#[test]
fn er_attributes_mono() {
    let src="erDiagram\n  CAR {\n    string registrationNumber PK \"The plate number, as issued by the registration office of the country\"\n    int wheels FK, UK\n  }";
    let plain = svg(src);
    let mono = svg_with(src, json!({"monospace":["attributes"]}));
    let comment = |s: &str| captures(s, r#"<text class="pele-er-comment".*?</text>"#)[0].clone();
    for cls in ["pele-er-type", "pele-er-name", "pele-er-keys"] {
        assert!(has(&mono, &format!(r#"<text class="{cls}"[^>]* font-family="var\(--_fm\)""#)));
    }
    assert!(!plain.contains("font-family=\"var(--_fm)\""));
    assert!(!comment(&mono).contains("font-family"));
    assert_eq!(re(r#" x="[-\d.]+""#).replace_all(&comment(&mono), ""), re(r#" x="[-\d.]+""#).replace_all(&comment(&plain), ""));
    assert_eq!(comment(&mono).matches("<tspan").count(), 2);
    let width = |s: &str| {
        number(
            &captures(
                s,
                r#"data-id="CAR" transform="translate\([-\d.]+,[-\d.]+\)"><rect x="[-\d.]+" y="[-\d.]+" width="([\d.]+)" height="[\d.]+""#,
            )[1],
        )
    };
    assert!(width(&mono) > width(&plain));
    for name in ["`a  b`", "`<code>a  b</code>`"] {
        let s = svg_with(&format!("erDiagram\n  A {{\n    string {name}\n  }}"), json!({"monospace":["attributes"]}));
        assert!(has(&s, r#"<text class="pele-er-name" font-family="var\(--_fm\)" xml:space="preserve"><tspan[^>]*>a {2}b</tspan>"#));
    }
}
#[cfg(feature = "er")]
#[test]
fn er_invalid_mono() {
    let src = "erDiagram\n  CAR {\n    string registrationNumber PK \"The plate\"\n  }";
    let plain = svg(src);
    for mono in
        [json!([]), json!(["attribute"]), json!(["members"]), json!("attributes"), json!({"attributes":true}), json!(true), Value::Null]
    {
        assert_eq!(svg_with(src, json!({"monospace":mono})), plain);
    }
}
#[cfg(feature = "er")]
#[test]
fn er_own_font() {
    for rule in ["style A color:black,font-family:serif", "classDef serif color:black,font-family:serif\n  class A serif"] {
        let src = format!("erDiagram\n  A {{\n    string name PK\n  }}\n  B {{\n    string name PK\n  }}\n  {rule}");
        let attributes = |s: &str| captures(s, r#"data-id="A".*?(<g class="pele-er-attributes".*?</g>)"#)[1].clone();
        let s = svg_with(&src, json!({"monospace":["attributes"]}));
        assert_eq!(attributes(&s), attributes(&svg(&src)));
        assert!(captures(&s, r#"data-id="B".*$"#)[0].contains("font-family=\"var(--_fm)\""));
    }
}
#[cfg(feature = "treeView")]
#[test]
fn tree_names_mono() {
    let src = "treeView-beta\n    src/\n        App.tsx ## main\n";
    let desc = |s: &str| number(&captures(s, r#"class="pele-tree-description" x="([\d.]+)""#)[1]);
    let plain = svg(src);
    for width in [Value::Null, json!(90)] {
        let s = svg_with(src, json!({"maxWidth":width,"monospace":["files"]}));
        assert_eq!(matches(&s, r#"<text class="pele-label"[^>]* font-family="var\(--_fm\)""#).len(), if width.is_null() { 2 } else { 3 });
        assert!(has(&s, r#"<text class="pele-label"[^>]* font-weight="var\(--_hw\)"[^>]*>src<"#));
        assert!(!has(&s, r#"<text class="pele-tree-description"[^>]* font-family="#));
    }
    assert!(!plain.contains("font-family=\""));
    assert!(desc(&svg_with(src, json!({"monospace":["files"]}))) > desc(&plain));
    for mono in [json!([]), json!(["file"]), json!(["members"]), json!("files")] {
        assert_eq!(svg_with(src, json!({"monospace":mono})), plain);
    }
}
