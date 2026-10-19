//! Raw flowchart parser through the frozen compatibility adapter's model projection.
use super::{graph, parser, FlowchartModel};
use crate::upstream_support::{self, Adapter};
use serde_json::{json, Value};
use std::sync::LazyLock;
struct FlowAdapter {
    db: FlowchartModel,
    observations: std::collections::HashMap<String, (usize, Option<usize>)>,
}
impl Default for FlowAdapter {
    fn default() -> Self {
        let mut model = FlowchartModel::default();
        model.max_edges = 500;
        model.collect_warnings = true;
        Self { db: model, observations: Default::default() }
    }
}
impl Adapter for FlowAdapter {
    fn reset(&mut self) {
        *self = Self::default()
    }
    fn parse(&mut self, source: &str) -> Result<(), String> {
        parser::parse(source, &mut self.db).map_err(|e| e.message)
    }
    fn observe(&mut self, method: &str, enabled: bool) {
        self.db.calls.get_or_insert_with(Vec::new);
        let len = if method == "warn" { self.db.warnings.len() } else { self.db.calls.as_ref().unwrap().len() };
        if enabled {
            self.observations.insert(method.into(), (len, None));
        } else {
            self.observations.get_mut(method).expect("active spy").1 = Some(len);
        }
    }
    fn root(&self, key: &str) -> Result<Option<Value>, String> {
        let db = &self.db;
        if let Some(method) = key.strip_prefix("calls:") {
            let (start, end) = self.observations.get(method).expect("observed method");
            let entries: Vec<Value> = if method == "warn" {
                db.warnings.iter().take(end.unwrap_or(db.warnings.len())).skip(*start).map(|s| json!([s])).collect()
            } else {
                let calls = db.calls.as_ref().expect("recording enabled");
                calls
                    .iter()
                    .take(end.unwrap_or(calls.len()))
                    .skip(*start)
                    .filter(|(name, _)| *name == method)
                    .map(|(_, args)| Value::Array(args.iter().map(|a| serde_json::from_str(&a.to_json_string()).unwrap()).collect()))
                    .collect()
            };
            return Ok(Some(json!(entries)));
        }
        let optional = |v: &mut Value, key: &str, s: &Option<String>| {
            if let Some(s) = s {
                v[key] = json!(s)
            }
        };
        Ok(Some(match key {
            "direction"=>return Ok(db.direction.as_ref().map(|s|json!(crate::util::js::trim(s)))),
            "defaultInterpolate"=>return Ok(db.default_interpolate.as_ref().map(|s|json!(s))),
            "defaultEdgeStyle"=>return Ok((!db.default_edge_style.is_empty()).then(||json!(db.default_edge_style))),
            "accTitle"=>json!(db.acc_title.as_deref().unwrap_or("")),
            "accDescr"=>json!(db.acc_descr.as_deref().unwrap_or("")),
            "tooltips"=>upstream_support::map(db.tooltips.iter().map(|(id,v)|(id.clone(),json!(v)))),
            "nodes"=>upstream_support::map(db.nodes.iter().map(|(id,n)|{
                let mut v=json!({"id":n.id,"labelType":n.label_type,"styles":n.styles,"classes":n.classes});
                for(k,s)in[("text",&n.text),("type",&n.shape),("dir",&n.dir),("link",&n.link),("linkTarget",&n.link_target),("icon",&n.icon),("form",&n.form),("pos",&n.pos),("img",&n.img),("constraint",&n.constraint)]{optional(&mut v,k,s)}
                if !n.props.is_empty(){v["props"]=json!(n.props)}
                if let Some(x)=n.asset_width{v["assetWidth"]=json!(x)}
                if let Some(x)=n.asset_height{v["assetHeight"]=json!(x)}
                (id.clone(),v)
            })),
            "edges"=>json!(db.edges.iter().map(|e|{
                let mut v=json!({"start":e.start,"end":e.end,"type":e.kind,"text":e.text,"labelType":e.label_type,"stroke":e.stroke,"classes":e.classes,"id":e.id,"isUserDefinedId":e.user_id});
                if let Some(n)=e.length {v["length"]=json!(n)}
                if !e.styles.is_empty(){v["style"]=json!(e.styles)}
                optional(&mut v,"interpolate",&e.interpolate);optional(&mut v,"animation",&e.animation);
                if let Some(x)=e.animate{v["animate"]=json!(x)}
                v
            }).collect::<Vec<_>>()),
            "classes"=>upstream_support::map(db.classes.iter().map(|(id,c)|(id.clone(),json!({"id":id,"styles":c.styles,"textStyles":c.text_styles})))),
            "subgraphs"=>json!(db.subgraphs.iter().map(|g|{let mut v=json!({"id":g.id,"nodes":g.nodes,"title":g.title,"classes":g.classes,"labelType":g.label_type});optional(&mut v,"dir",&g.dir);if !g.metadata.is_empty(){v["metadata"]=serde_json::from_str(&crate::ConfigValue::Object(g.metadata.clone()).to_json_string()).unwrap()}v}).collect::<Vec<_>>()),
            "data"=>{
                let g=graph::build(db,None);
                let nodes:Vec<_>=g.nodes.iter().map(|n|{
                    let mut v=json!({"id":n.id,"labelType":n.label_type,"shape":n.shape,"isGroup":n.is_group,"cssStyles":n.styles,"cssCompiledStyles":n.compiled,"cssClasses":n.classes});
                    for(k,s)in[("label",&n.label),("parentId",&n.parent),("dir",&n.dir),("link",&n.link),("linkTarget",&n.link_target),("tooltip",&n.tooltip),("icon",&n.icon),("pos",&n.pos),("img",&n.img),("constraint",&n.constraint)]{optional(&mut v,k,s)}
                    if let Some(x)=n.asset_width{v["assetWidth"]=json!(x)}
                    if let Some(x)=n.asset_height{v["assetHeight"]=json!(x)}v
                }).collect();
                let edges:Vec<_>=g.edges.iter().map(|e|{let mut v=json!({"id":e.id,"start":e.start,"end":e.end,"label":e.label,"labelType":e.label_type,"thickness":e.thickness,"arrowTypeStart":e.arrow_start,"arrowTypeEnd":e.arrow_end,"cssCompiledStyles":e.compiled,"style":e.styles,"classes":e.classes,"isUserDefinedId":e.user_id});optional(&mut v,"curve",&e.curve);optional(&mut v,"animation",&e.animation);if let Some(a)=e.animate{v["animate"]=json!(a)}if let Some(x)=e.minlen{v["minlen"]=json!(x)}v}).collect();
                json!({"nodes":nodes,"edges":edges,"other":{},"config":{"securityLevel":"strict"}})
            },
            _=>return Err(format!("unmapped FlowDB root {key}")),
        }))
    }
}
fn run_case(id: &str) {
    static DATA: LazyLock<Value> =
        LazyLock::new(|| serde_json::from_str(include_str!("../../../../tests/upstream/ir/flow-parser.json")).unwrap());
    let r = DATA["records"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap();
    let result = upstream_support::execute(r["actions"].as_array().unwrap(), &mut FlowAdapter::default());
    assert!(result.is_ok(), "{}: {}", r["name"], result.unwrap_err());
}
include!("flow_parser_generated.rs");
