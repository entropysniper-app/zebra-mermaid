//! Raw ER parser/DB projection corresponding to Pele tests/compat/er/adapter.ts.
use super::{parser, ErModel};
use crate::upstream_support::{self, Adapter};
use serde_json::{json, Value};
use std::sync::LazyLock;
struct ErAdapter(ErModel);
impl Adapter for ErAdapter {
    fn reset(&mut self) {
        self.0 = ErModel::default();
    }
    fn parse(&mut self, source: &str) -> Result<(), String> {
        parser::parse(source, &mut self.0).map_err(|e| e.message)
    }
    fn root(&self, key: &str) -> Result<Value, String> {
        let db = &self.0;
        Ok(match key{
  "entities"=>upstream_support::map(db.entities.iter().map(|(k,e)|(k.clone(),json!({"id":e.id,"label":e.label,"alias":e.alias,"cssClasses":e.classes,"cssStyles":e.styles,"attributes":e.attributes.iter().map(|a|json!({"type":a.kind,"name":a.name,"keys":a.keys,"comment":a.comment})).collect::<Vec<_>>()})))),
  "relationships"=>json!(db.relationships.iter().map(|r|json!({"entityA":r.a,"entityB":r.b,"roleA":r.role,"relSpec":{"cardA":r.card_a.to_uppercase(),"cardB":r.card_b.to_uppercase(),"relType":if r.identifying{"IDENTIFYING"}else{"NON_IDENTIFYING"}}})).collect::<Vec<_>>()),
  "classes"=>upstream_support::map(db.classes.iter().map(|(id,c)|(id.clone(),json!({"id":id,"styles":c.styles,"textStyles":c.text_styles})))),
  "accTitle"=>json!(db.acc_title.as_deref().unwrap_or("")),"accDescr"=>json!(db.acc_descr.as_deref().unwrap_or("")),
  _=>return Err(format!("unknown ER adapter root {key}")),
 })
    }
}
fn run_case(id: &str) {
    static DATA: LazyLock<Value> =
        LazyLock::new(|| serde_json::from_str(include_str!("../../../../tests/upstream/ir/er-parser.json")).unwrap());
    let record = DATA["records"].as_array().unwrap().iter().find(|r| r["id"] == id).expect("mapped original test");
    let result = upstream_support::execute(record["actions"].as_array().unwrap(), &mut ErAdapter(ErModel::default()));
    assert!(result.is_ok(), "{}: {}", record["name"], result.unwrap_err());
}
include!("er_generated.rs");
