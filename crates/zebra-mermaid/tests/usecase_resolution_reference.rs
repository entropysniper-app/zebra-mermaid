//! Supplementary fixed-Pele semantic regression; not an original-test acceptance mapping.
#![cfg(feature = "usecase")]
mod common;
use flate2::read::GzDecoder;
use serde_json::{json, Value};
use std::io::Read;
use zebra_mermaid::{parse, DiagramModel, ParseOptions};

fn model_value(m: &zebra_mermaid::diagrams::usecase::UsecaseModel) -> Value {
    let actors: Vec<_> = m
        .actors
        .iter()
        .map(|(id, e)| {
            json!([id, {
                "id":e.id,"label":e.label,"labelType":e.label_type,"kind":e.kind,"shape":e.shape,
                "icon":e.icon,"business":e.business,"stereotype":e.stereotype,"parent":e.parent,
                "classes":e.classes,"styles":e.styles
            }])
        })
        .collect();
    let use_cases: Vec<_> = m
        .use_cases
        .iter()
        .map(|(id, e)| {
            json!([id, {
                "id":e.id,"label":e.label,"labelType":e.label_type,"kind":e.kind,"shape":e.shape,
                "icon":e.icon,"business":e.business,"stereotype":e.stereotype,"parent":e.parent,
                "classes":e.classes,"styles":e.styles
            }])
        })
        .collect();
    let boundaries: Vec<_> = m
        .boundaries
        .iter()
        .map(|(id, b)| {
            json!([id,{
                "id":b.id,"label":b.label,"labelType":b.label_type,"kind":b.kind,
                "members":b.members,"classes":b.classes,"styles":b.styles
            }])
        })
        .collect();
    let relationships: Vec<_> = m
        .relationships
        .iter()
        .map(|e| {
            json!({
                "id":e.id,"explicitId":e.explicit_id,"source":e.source,"target":e.target,"kind":e.kind,
                "arrowType":e.arrow_type,"label":e.label,"labelType":e.label_type,"minlen":e.minlen,
                "classes":e.classes,"styles":e.styles,"animate":e.animate,"animation":e.animation
            })
        })
        .collect();
    let notes: Vec<_> = m
        .notes
        .iter()
        .map(|(id, n)| {
            json!([id,{
                "id":n.id,"target":n.target,"label":n.label,"labelType":n.label_type
            }])
        })
        .collect();
    let json_nodes: Vec<_> = m
        .json_nodes
        .iter()
        .map(|(id, n)| {
            let order: Vec<_> = n.value.property_order.iter().map(|(key, value)| json!([key, value])).collect();
            json!([id,{"id":n.id,"classes":n.classes,"styles":n.styles,"propertyOrder":order}])
        })
        .collect();
    let class_defs: Vec<_> = m.class_defs.iter().map(|(id, styles)| json!([id, styles])).collect();
    json!({"actors":actors,"useCases":use_cases,"boundaries":boundaries,"relationships":relationships,
        "notes":notes,"jsonNodes":json_nodes,"classDefs":class_defs,"direction":m.direction,"title":m.title,
        "accTitle":m.acc_title,"accDescr":m.acc_descr,"accDescription":m.acc_description})
}

#[test]
fn semantic_order_metadata_and_error_precedence_match_pinned_node() {
    let mut source = String::new();
    GzDecoder::new(include_bytes!("../../../tests/fixtures/unit/usecase-resolution-reference.json.gz").as_slice())
        .read_to_string(&mut source)
        .unwrap();
    let fixture: Value = serde_json::from_str(&source).unwrap();
    assert_eq!(fixture["upstreamCommit"], zebra_mermaid::PELE_COMMIT);
    assert_eq!(fixture["seed"], "0x4a0e14");
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 161);
    let mut successes = 0;
    for row in cases {
        let actual = match parse(row["source"].as_str().unwrap(), &ParseOptions::default()) {
            Ok(parsed) => {
                let DiagramModel::Usecase(model) = parsed.model() else { panic!("wrong model kind") };
                successes += 1;
                json!({"ok":true,"model":model_value(model)})
            }
            Err(e) => json!({"ok":false,"error":{"code":e.code.as_str(),"type":e.diagram_type.map(|t|t.as_str()),
                "mermaidType":e.mermaid_type.map(|t|t.as_str()),"line":e.line,"column":e.column,
                "snippet":e.snippet,"message":e.message}}),
        };
        common::json_eq(&actual, &row["expect"], &format!("usecase-resolution-{}", row["index"])).unwrap();
    }
    assert_eq!(successes, 132);
    assert_eq!(cases.len() - successes, 29);
}
