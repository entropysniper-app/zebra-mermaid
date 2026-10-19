//! Raw ClassModel projection through the original ClassDB adapter contract.
use super::{parser, ClassModel, ClassNode, Member, Note};
use crate::upstream_support::{self, Adapter};
use serde_json::{json, Value};
use std::sync::LazyLock;
fn optional(v: &mut Value, key: &str, value: &Option<String>) {
    if let Some(s) = value {
        v[key] = json!(s)
    }
}
fn member(m: &Member) -> Value {
    json!({"id":m.id,"memberType":if m.method{"method"}else{"attribute"},"visibility":m.visibility,"parameters":m.parameters,"classifier":m.classifier,"returnType":m.return_type,
        "displayDetails":{"displayText":m.display(),"cssStyle":match m.classifier.as_str(){"*"=>"font-style:italic;","$"=>"text-decoration:underline;",_=>""}}})
}
fn class(c: &ClassNode) -> Value {
    let mut v = json!({"id":c.id,"type":c.kind,"label":c.label,"cssClasses":c.css_classes,"methods":c.methods.iter().map(member).collect::<Vec<_>>(),"members":c.members.iter().map(member).collect::<Vec<_>>(),"annotations":c.annotations,"styles":c.styles});
    for (k, s) in [("parent", &c.parent), ("link", &c.link), ("linkTarget", &c.link_target), ("tooltip", &c.tooltip)] {
        optional(&mut v, k, s)
    }
    if c.callback {
        v["haveCallback"] = json!(true)
    }
    v
}
fn note(n: &Note) -> Value {
    let mut v = json!({"id":n.id,"text":n.text,"index":n.index});
    optional(&mut v, "class", &n.class);
    optional(&mut v, "parent", &n.parent);
    v
}
fn namespace(db: &ClassModel, id: &str) -> Value {
    let n = &db.namespaces[id];
    let mut v = json!({"id":n.id,"label":n.label,"explicit":n.explicit,
        "classes":upstream_support::map(db.classes.iter().filter(|(_,c)|c.parent.as_deref()==Some(id)).map(|(id,c)|(id.clone(),class(c)))),
        "notes":upstream_support::map(db.notes.iter().filter(|(_,n)|n.parent.as_deref()==Some(id)).map(|(id,n)|(id.clone(),note(n)))),
        "children":upstream_support::map(db.namespaces.iter().filter(|(_,n)|n.parent.as_deref()==Some(id)).map(|(id,_)|(id.clone(),namespace(db,id))))});
    optional(&mut v, "parent", &n.parent);
    v
}
struct ClassAdapter(ClassModel);
impl Adapter for ClassAdapter {
    fn reset(&mut self) {
        self.0 = ClassModel::default()
    }
    fn parse(&mut self, source: &str) -> Result<(), String> {
        parser::parse(source, &mut self.0).map_err(|e| e.message)
    }
    fn observe(&mut self, method: &str, enabled: bool) {
        assert!(enabled && ["setLink", "setTooltip", "setClickEvent"].contains(&method), "unsupported class spy");
        self.0.calls.get_or_insert_with(Vec::new).retain(|(name, _)| *name != method);
    }
    fn root(&self, key: &str) -> Result<Option<Value>, String> {
        let db = &self.0;
        if let Some(method) = key.strip_prefix("calls:") {
            return Ok(Some(json!(db
                .calls
                .as_ref()
                .expect("active spy")
                .iter()
                .filter(|(name, _)| *name == method)
                .map(|(_, args)| args)
                .collect::<Vec<_>>())));
        }
        Ok(Some(match key{
            "classes"=>upstream_support::map(db.classes.iter().map(|(id,c)|(id.clone(),class(c)))),
            "notes"=>upstream_support::map(db.notes.iter().map(|(id,n)|(id.clone(),note(n)))),
            "namespaces"=>upstream_support::map(db.namespaces.keys().map(|id|(id.clone(),namespace(db,id)))),
            "relations"=>json!(db.relations.iter().map(|r|{let mut v=json!({"id1":r.id1,"id2":r.id2,"relationTitle1":r.title1,"relationTitle2":r.title2,"relation":{"type1":r.end1.map_or(json!("none"),|n|json!(n)),"type2":r.end2.map_or(json!("none"),|n|json!(n)),"lineType":usize::from(r.dotted)}});optional(&mut v,"title",&r.title);v}).collect::<Vec<_>>()),
            "direction"=>json!(db.direction),"accTitle"=>json!(db.acc_title.as_deref().unwrap_or("")),"accDescr"=>json!(db.acc_descr.as_deref().unwrap_or("")),
            _=>return Err(format!("unmapped ClassDB root {key}")),
        }))
    }
}
fn run_case(id: &str) {
    static DATA: LazyLock<Value> =
        LazyLock::new(|| serde_json::from_str(include_str!("../../../../tests/upstream/ir/class-parser.json")).unwrap());
    let r = DATA["records"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap();
    let result = upstream_support::execute(r["actions"].as_array().unwrap(), &mut ClassAdapter(ClassModel::default()));
    assert!(result.is_ok(), "{}: {}", r["name"], result.unwrap_err());
}
include!("class_parser_generated.rs");
