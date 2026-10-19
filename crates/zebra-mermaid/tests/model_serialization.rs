//! Preserve the public serialized model while changing its internal storage.
#[cfg(all(feature = "class", feature = "serde"))]
#[test]
fn class_style_lists_retain_duplicates_and_statement_order() {
    let source =
        "classDiagram\nclass A:::k\nclass B:::k\nclassDef k fill:red\nclassDef k fill:red\nstyle A stroke:blue\nclassDef k color:green\n";
    let parsed = zebra_mermaid::parse(source, &Default::default()).unwrap();
    let zebra_mermaid::DiagramModel::Class(model) = parsed.model() else { panic!("class model") };
    let value = serde_json::to_value(model).unwrap();
    assert_eq!(value["classes"]["A"]["styles"], serde_json::json!(["fill:red", "fill:red", "stroke:blue", "color:green"]));
    assert_eq!(value["classes"]["B"]["styles"], serde_json::json!(["fill:red", "fill:red", "color:green"]));
}

#[cfg(all(feature = "agentflow", feature = "serde"))]
#[test]
fn global_blocks_retain_serialized_membership_order() {
    let source = "agentflow-beta TB\nflow f\nZ\nA\nB\nend\nglobal\nZ\nend\nflow g\nC\nD\nend\nglobal\nB\nend\n";
    let parsed = zebra_mermaid::parse(source, &Default::default()).unwrap();
    let zebra_mermaid::DiagramModel::Agentflow(model) = parsed.model() else { panic!("agentflow model") };
    let value = serde_json::to_value(model).unwrap();
    assert_eq!(
        value.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "direction",
            "nodes",
            "edges",
            "subgraphs",
            "connectors",
            "classes",
            "tooltips",
            "default_edge_style",
            "default_interpolate",
            "mappings",
            "diagnostics",
            "title",
            "acc_title",
            "acc_descr",
            "line_offset",
            "graph",
            "max_edges",
            "inherit_dir",
            "configured_direction",
            "lookup",
            "global_nodes",
            "members",
            "edge_by_id",
            "pair_count",
            "sub_count",
            "bare_mappings",
            "first_mapping",
        ]
    );
    assert_eq!(value["members"], serde_json::json!(["A", "C", "D"]));
    assert_eq!(value["global_nodes"], serde_json::json!(["Z", "B"]));
    assert_eq!(value["subgraphs"][0]["nodes"], serde_json::json!(["A"]));
    assert_eq!(value["subgraphs"][1]["nodes"], serde_json::json!(["C", "D"]));
}

#[cfg(all(feature = "agentflow", feature = "serde"))]
#[test]
fn global_blocks_release_multiple_owners_and_reopened_nested_flows_in_order() {
    let source = "agentflow-beta TB\nflow outer\nO\nflow inner\nA\nB\nC\nend\nP\nend\nflow other\nD\nE\nF\nend\nglobal\nB\nE\nB\nX\nend\nflow inner\nG\nB\nH\nend\nglobal\nG\nend\nglobal\nD\nF\nC\nend\n";
    let parsed = zebra_mermaid::parse(source, &Default::default()).unwrap();
    let zebra_mermaid::DiagramModel::Agentflow(model) = parsed.model() else { panic!("agentflow model") };
    let value = serde_json::to_value(model).unwrap();
    assert_eq!(value["members"], serde_json::json!(["A", "O", "inner", "P", "H"]));
    assert_eq!(value["global_nodes"], serde_json::json!(["B", "E", "X", "G", "D", "F", "C"]));
    let groups: Vec<_> = value["subgraphs"].as_array().unwrap().iter().map(|s| (&s["id"], &s["nodes"])).collect();
    assert_eq!(
        serde_json::to_value(groups).unwrap(),
        serde_json::json!([["inner", ["A", "H"]], ["outer", ["O", "inner", "P"]], ["other", []]])
    );
}
