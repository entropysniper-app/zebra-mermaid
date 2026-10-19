//! The original sequence adapter's model shape, with one independent native model per Diagram.
use super::{db::SeqDb, parser, SeqBox, SeqContent, SequenceModel};
use crate::{
    upstream_support::{self, Adapter},
    Config, ConfigValue,
};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::LazyLock};
fn optional(v: &mut Value, key: &str, s: &Option<String>) {
    if let Some(s) = s {
        v[key] = json!(s)
    }
}
fn box_value(b: &SeqBox) -> Value {
    let mut v = json!({"wrap":b.wrap,"fill":b.fill,"actorKeys":b.actor_keys});
    optional(&mut v, "name", &b.name);
    v
}
fn config_value(v: &ConfigValue) -> Value {
    serde_json::from_str(&v.to_json_string()).unwrap()
}
#[derive(Default)]
struct SequenceAdapter {
    models: HashMap<usize, (SequenceModel, Config)>,
    views: HashMap<usize, Value>,
    results: HashMap<usize, Value>,
    numbering: HashMap<usize, super::render::Numbering>,
}
impl Adapter for SequenceAdapter {
    fn reset(&mut self) {
        self.models.clear();
        self.views.clear();
        self.results.clear();
        self.numbering.clear();
    }
    fn parse(&mut self, _: &str) -> Result<(), String> {
        panic!("sequence needs an explicit Diagram identity")
    }
    fn invoke(&mut self, action: &Value) -> Result<(), String> {
        let id = action["id"].as_u64().unwrap_or(0) as usize;
        match action["method"].as_str().expect("model operation") {
            "captureFailure" => {
                let result = upstream_support::execute(action["actions"].as_array().expect("try body"), self);
                let value = match action["kind"].as_str().expect("catch projection") {
                    "boolean" => json!(result.is_err()),
                    "message" => json!(result.err().unwrap_or_default()),
                    other => panic!("unsupported catch assignment {other}"),
                };
                self.results.insert(action["result"].as_u64().unwrap() as usize, value);
            }
            "parsePublic" => {
                let pre = crate::preprocess::preprocess(action["source"].as_str().unwrap()).map_err(|e| e.message)?;
                if crate::detect::detect(&pre.text) != Some(crate::DiagramType::Sequence) {
                    return Err("Not a sequence diagram.".into());
                }
                let mut config = crate::config::json::parse_json(&action["config"].to_string()).unwrap().as_object().unwrap().clone();
                // The original adapter spreads the preprocessor configuration at the top level.
                for (key, value) in pre.config.iter() {
                    config.insert(key, value.clone());
                }
                let source = crate::preprocess::encode_entities(&pre.text).0 + "\n";
                super::parse(&crate::diagrams::ParseInput {
                    source: &source,
                    config: &config,
                    title: pre.title.as_deref(),
                    line_offset: 0,
                    limits: crate::options::ParseLimits { max_edges: 500 },
                })
                .map_err(|e| e.message)?;
                self.results.insert(action["result"].as_u64().unwrap() as usize, json!({"diagramType":"sequence"}));
            }
            "render" => {
                let (model, _) = self.models.get(&id).expect("existing Diagram");
                let config = crate::config::json::parse_json(&action["config"].to_string()).unwrap().as_object().unwrap().clone();
                let options = crate::RenderOptions::default();
                let theming = crate::theme::Theming::new(options.color_mode, options.theme, options.palette.as_ref(), options.canvas());
                let icons = crate::text::Icons::none();
                let context = crate::diagrams::RenderCtx::new(&options, true, &crate::text::MetricsMeasurer, &icons, &theming);
                super::render::render(model, &config, &context).map_err(|e| e.message)?;
                self.numbering.insert(id, super::render::number_messages(&model.messages, false));
            }
            "actorView" => {
                let value = self.root(&format!("{id}:actors"))?.expect("actor view");
                self.views.insert(action["view"].as_u64().unwrap() as usize, value);
            }
            "setViewDescription" => {
                let view = self.views.get_mut(&(action["view"].as_u64().unwrap() as usize)).expect("actor view");
                assert_eq!(action["actor"]["op"], "literal");
                let actor = &action["actor"]["value"];
                let target =
                    view["$map"].as_array_mut().unwrap().iter_mut().find(|a| &a[0] == actor).ok_or("Cannot set properties of undefined")?;
                target[1]["description"] = action["value"].clone();
            }
            "create" => {
                let cfg = crate::config::json::parse_json(&action["config"].to_string()).unwrap().as_object().unwrap().clone();
                self.models.insert(id, (SequenceModel::default(), cfg));
            }
            "parseDiagram" => {
                let source = action["source"].as_str().unwrap();
                let pre = crate::preprocess::preprocess(source).map_err(|e| e.message)?;
                if crate::detect::detect(&pre.text) != Some(crate::DiagramType::Sequence) {
                    return Err("Not a sequence diagram.".into());
                }
                let (_, cfg) = self.models.get(&id).expect("created model");
                let mut db = SeqDb::new(
                    cfg.get("wrap").and_then(ConfigValue::as_bool),
                    cfg.lookup("sequence", "wrap").and_then(ConfigValue::as_bool) == Some(true),
                );
                let text = crate::preprocess::encode_entities(source).0 + "\n";
                parser::parse_sequence(&text, &mut db).map_err(|e| e.message)?;
                self.models.get_mut(&id).unwrap().0 = db.finish();
            }
            method => panic!("unsupported sequence operation {method}"),
        }
        Ok(())
    }
    fn root(&self, key: &str) -> Result<Option<Value>, String> {
        if let Some(id) = key.strip_prefix("result:") {
            return Ok(Some(self.results.get(&id.parse().unwrap()).expect("existing operation result").clone()));
        }
        if let Some(id) = key.strip_prefix("view:") {
            return Ok(Some(self.views.get(&id.parse().unwrap()).expect("existing actor view").clone()));
        }
        let (id, key) = key.split_once(':').expect("Diagram root identity");
        let id = id.parse().unwrap();
        let (m, _) = self.models.get(&id).expect("existing Diagram");
        Ok(Some(match key {
            "numbersShown" => json!(self.numbering.get(&id).is_some_and(|n| n.visible)),
            "actors" => upstream_support::map(m.actors.iter().map(|(id, a)| {
                let mut v = json!({"name":a.name,"description":a.description,"wrap":a.wrap,"type":a.kind,
                    "links":a.links.iter().map(|(k,v)|(k.clone(),config_value(v))).collect::<serde_json::Map<_,_>>(),
                    "properties":a.properties.iter().map(|(k,v)|(k.clone(),config_value(v))).collect::<serde_json::Map<_,_>>()});
                optional(&mut v, "prevActor", &a.prev_actor);
                optional(&mut v, "nextActor", &a.next_actor);
                optional(&mut v, "details", &a.details);
                if let Some(i) = a.box_index {
                    v["box"] = box_value(&m.boxes[i]);
                }
                (id.clone(), v)
            })),
            "actorKeys" => json!(m.actors.keys().collect::<Vec<_>>()),
            "boxes" => json!(m.boxes.iter().map(box_value).collect::<Vec<_>>()),
            "createdActors" => upstream_support::map(m.created_actors.iter().map(|(k, v)| (k.clone(), json!(v)))),
            "destroyedActors" => upstream_support::map(m.destroyed_actors.iter().map(|(k, v)| (k.clone(), json!(v)))),
            "messages" => json!(m
                .messages
                .iter()
                .enumerate()
                .map(|(index, msg)| {
                    let content = match &msg.message {
                        SeqContent::Text(s) => json!(s),
                        SeqContent::Numbering { start, step, visible } => {
                            let mut v = json!({"visible":visible});
                            if let Some(n) = start {
                                v["start"] = json!(n)
                            }
                            if let Some(n) = step {
                                v["step"] = json!(n)
                            }
                            v
                        }
                    };
                    let mut v = json!({"id":msg.id.to_string(),"message":content,"wrap":msg.wrap,"type":msg.kind});
                    optional(&mut v, "from", &msg.from);
                    optional(&mut v, "to", &msg.to);
                    if let Some(a) = msg.activate {
                        v["activate"] = json!(a)
                    }
                    if let Some(a) = msg.placement {
                        v["placement"] = json!(a)
                    }
                    if let Some(a) = msg.central_connection {
                        v["centralConnection"] = json!(a)
                    }
                    if let Some(numbering) = self.numbering.get(&id) {
                        v["msgModel"] = if numbering.index[index].is_nan() {
                            json!({})
                        } else {
                            json!({"sequenceIndex":numbering.index[index],"sequenceVisible":numbering.shown[index]})
                        };
                    }
                    v
                })
                .collect::<Vec<_>>()),
            "title" => json!(m.title.as_deref().unwrap_or("")),
            "accTitle" => json!(m.acc_title.as_deref().unwrap_or("")),
            "accDescr" => json!(m.acc_descr.as_deref().unwrap_or("")),
            _ => return Err(format!("unmapped sequence observation {key}")),
        }))
    }
}
fn run_case(id: &str) {
    static DATA: LazyLock<Value> =
        LazyLock::new(|| serde_json::from_str(include_str!("../../../../tests/upstream/ir/sequence-parser.json")).unwrap());
    let r = DATA["records"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap();
    let mut adapter = SequenceAdapter::default();
    upstream_support::execute(r["setupActions"].as_array().unwrap(), &mut adapter).expect("original beforeEach");
    upstream_support::with_timeout(r["timeout"].as_u64().expect("original test timeout"), move || {
        let result = upstream_support::execute(r["actions"].as_array().unwrap(), &mut adapter);
        assert!(result.is_ok(), "{}: {}", r["name"], result.unwrap_err());
    });
}
include!("sequence_parser_generated.rs");
