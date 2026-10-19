#![cfg(feature = "state")]

use zebra_mermaid::{parse, DiagramModel, ErrorCode};

#[test]
fn repeated_descriptions_keep_order_across_notes() {
    let parsed = parse("stateDiagram-v2\na: Heading\na: first\na: second\nnote right of a: note\na: third\n", &Default::default()).unwrap();
    let DiagramModel::State(model) = parsed.model() else { panic!("state model") };
    assert_eq!(model.graph.nodes.iter().map(|node| node.id.as_str()).collect::<Vec<_>>(), ["a", "a----parent", "a----note-0"]);
    let node = &model.graph.nodes[0];
    assert_eq!(node.label.as_deref(), Some("Heading"));
    assert_eq!(node.shape.as_deref(), Some("rectWithTitle"));
    assert_eq!(node.description, ["first", "second", "third"]);
    assert!(model.graph.nodes[1].description.is_empty());
    assert!(model.graph.nodes[2].description.is_empty());
}

#[test]
fn note_group_replaces_an_existing_states_description() {
    let parsed = parse(
        "stateDiagram-v2\nstate \"Custom\" as a----parent\nstate \"Details\" as a----parent\nnote right of a: note\n",
        &Default::default(),
    )
    .unwrap();
    let DiagramModel::State(model) = parsed.model() else { panic!("state model") };
    let node = &model.graph.nodes[0];
    assert_eq!(node.id, "a----parent");
    assert_eq!(node.label.as_deref(), Some("note"));
    assert_eq!(node.shape.as_deref(), Some("noteGroup"));
    assert!(node.group);
    assert!(node.description.is_empty());
}

#[test]
fn state_description_replaces_an_existing_note_group() {
    let parsed = parse(
        "stateDiagram-v2\nnote right of a: note\nstate \"Custom\" as a----parent\nstate \"Details\" as a----parent\n",
        &Default::default(),
    )
    .unwrap();
    let DiagramModel::State(model) = parsed.model() else { panic!("state model") };
    let node = &model.graph.nodes[0];
    assert_eq!(node.id, "a----parent");
    assert_eq!(node.label.as_deref(), Some("Custom"));
    assert_eq!(node.shape.as_deref(), Some("rectWithTitle"));
    assert!(!node.group);
    assert_eq!(node.description, ["Details"]);
}

#[test]
fn group_descriptions_still_report_the_original_semantic_error() {
    let error = parse("stateDiagram-v2\nstate Container {\na\n}\nContainer: Title\nContainer: details\n", &Default::default()).unwrap_err();
    assert_eq!(error.code, ErrorCode::Semantic);
    assert_eq!(error.message, "Group nodes can only have label. Remove the additional description for node [Container]");
}
