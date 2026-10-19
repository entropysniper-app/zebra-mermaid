//! 1:1 port of the three ClassDB getData namespace tests, from Mermaid 12.1.0 (MIT).
//! Each test owns the configuration that the upstream adapter sets and resets in finally.
use super::{graph, parser, ClassModel};
fn parse(source: &str) -> ClassModel {
    let mut model = ClassModel::default();
    parser::parse(source, &mut model).unwrap();
    model
}
fn group_ids(nodes: &[graph::Node<'_>]) -> Vec<String> {
    let mut ids = nodes.iter().filter(|n| n.group).map(|n| n.id.clone()).collect::<Vec<_>>();
    ids.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
    ids
}
fn parent_id<'a>(nodes: &'a [graph::Node<'_>], node: &graph::Node<'_>) -> Option<&'a str> {
    usize::try_from(node.parent).ok().and_then(|index| nodes.get(index)).map(|n| n.id.as_str())
}
#[test]
fn case_004() {
    let model = parse(SOURCE_004);
    let (nodes, _) = graph::build(&model, false);
    assert_eq!(group_ids(&nodes), ["Company", "Company.Engineering.Backend"]);
    let backend = nodes.iter().filter(|n| n.group).find(|n| n.id == "Company.Engineering.Backend").unwrap();
    assert_eq!(backend.label, "Company.Engineering.Backend");
    assert_eq!(parent_id(&nodes, backend), None);
    let developer = nodes.iter().find(|n| n.id == "Developer").unwrap();
    assert_eq!(parent_id(&nodes, developer), Some("Company.Engineering.Backend"));
    let ceo = nodes.iter().find(|n| n.id == "CEO").unwrap();
    assert_eq!(parent_id(&nodes, ceo), Some("Company"));
}
#[test]
fn case_005() {
    let model = parse(SOURCE_005);
    assert!(!model.namespaces["Company.Engineering"].explicit);
    let (nodes, _) = graph::build(&model, false);
    assert_eq!(group_ids(&nodes), ["Company", "Company.Engineering.Backend"]);
}
#[test]
fn case_006() {
    let model = parse(SOURCE_006);
    // Original getData uses config.class?.hierarchicalNamespaces ?? true.
    let config = crate::Config::new();
    let hierarchical = config.lookup("class", "hierarchicalNamespaces").and_then(crate::ConfigValue::as_bool).unwrap_or(true);
    let (nodes, _) = graph::build(&model, hierarchical);
    assert_eq!(group_ids(&nodes), ["A", "A.B"]);
    let leaf = nodes.iter().find(|n| n.id == "A.B").unwrap();
    assert_eq!(leaf.label, "B");
    assert_eq!(parent_id(&nodes, leaf), Some("A"));
}

const SOURCE_004: &str = r#"classDiagram
          namespace Company.Engineering.Backend {
            class Developer {
              +writeCode()
            }
          }
          namespace Company {
            class CEO {
              +makeDecisions()
            }
          }"#;

const SOURCE_005: &str = r#"classDiagram
          namespace Company.Engineering.Backend {
            class Developer
          }
          namespace Company {
            class CEO
          }"#;

const SOURCE_006: &str = r#"classDiagram
        namespace A.B {
          class Foo
        }"#;
