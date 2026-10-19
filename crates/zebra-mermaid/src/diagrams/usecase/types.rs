//! Use-case semantic data and source occurrences. Statement nesting uses arena indexes.
use crate::ConfigValue;
use indexmap::IndexMap;
pub type Span = [usize; 2];
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Element {
    pub id: String,
    pub label: String,
    pub label_type: String,
    pub kind: String,
    pub shape: String,
    pub icon: Option<String>,
    pub business: bool,
    pub stereotype: Option<String>,
    pub parent: Option<String>,
    pub classes: Vec<String>,
    pub styles: Vec<String>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Boundary {
    pub id: String,
    pub label: String,
    pub label_type: String,
    pub kind: String,
    pub members: Vec<String>,
    pub classes: Vec<String>,
    pub styles: Vec<String>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Relationship {
    pub id: String,
    pub explicit_id: bool,
    pub source: String,
    pub target: String,
    pub kind: String,
    pub arrow_type: u8,
    pub label: Option<String>,
    pub label_type: Option<String>,
    pub minlen: f64,
    pub classes: Vec<String>,
    pub styles: Vec<String>,
    pub animate: bool,
    pub animation: Option<String>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Note {
    pub id: String,
    pub target: String,
    pub label: String,
    pub label_type: String,
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum JsonValue {
    Scalar(ConfigValue),
    Array(Vec<usize>),
    Object(IndexMap<String, usize>),
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct OrderedJson {
    pub values: Vec<JsonValue>,
    pub root: usize,
    pub property_order: IndexMap<String, Vec<String>>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct JsonNode {
    pub id: String,
    pub value: OrderedJson,
    pub classes: Vec<String>,
    pub styles: Vec<String>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MetadataOccurrence {
    pub key: String,
    pub span: Span,
    pub key_span: Span,
    pub value_span: Span,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct NodeOccurrence {
    pub id: String,
    pub span: Span,
    pub id_span: Span,
    pub label_span: Option<Span>,
    pub defines: bool,
    pub stereotype_span: Option<Span>,
    pub metadata: Option<Vec<MetadataOccurrence>>,
    pub class_spans: Vec<Span>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EdgeOccurrence {
    pub id: String,
    pub span: Span,
    pub label_span: Option<Span>,
    pub id_span: Option<Span>,
    pub metadata: Option<Vec<MetadataOccurrence>>,
}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Statement {
    pub kind: String,
    pub span: Span,
    pub nodes: Option<Vec<NodeOccurrence>>,
    pub edges: Option<Vec<EdgeOccurrence>>,
    pub group: Option<String>,
    pub id_span: Option<Span>,
    pub title_span: Option<Span>,
    pub end_span: Option<Span>,
    pub reference: Option<String>,
    pub ref_span: Option<Span>,
    pub children: Vec<usize>,
    pub metadata: Option<Vec<MetadataOccurrence>>,
    pub class_spans: Vec<Span>,
}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UsecaseModel {
    pub actors: IndexMap<String, Element>,
    pub use_cases: IndexMap<String, Element>,
    pub boundaries: IndexMap<String, Boundary>,
    pub relationships: Vec<Relationship>,
    pub notes: IndexMap<String, Note>,
    pub json_nodes: IndexMap<String, JsonNode>,
    pub class_defs: IndexMap<String, Vec<String>>,
    pub direction: String,
    pub title: Option<String>,
    pub acc_title: String,
    pub acc_descr: String,
    pub acc_description: String,
    pub source: String,
    pub header_span: Span,
    pub statements: Vec<Statement>,
    pub roots: Vec<usize>,
}
impl Default for UsecaseModel {
    fn default() -> Self {
        Self {
            actors: IndexMap::new(),
            use_cases: IndexMap::new(),
            boundaries: IndexMap::new(),
            relationships: vec![],
            notes: IndexMap::new(),
            json_nodes: IndexMap::new(),
            class_defs: IndexMap::new(),
            direction: "LR".into(),
            title: None,
            acc_title: String::new(),
            acc_descr: String::new(),
            acc_description: String::new(),
            source: String::new(),
            header_span: [0, 0],
            statements: vec![],
            roots: vec![],
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Label {
    pub text: String,
    pub kind: String,
    pub span: Span,
}
#[derive(Clone, Debug, Default)]
pub struct Property {
    pub occurrence: MetadataOccurrence,
    pub value: ConfigValue,
}
#[derive(Clone, Debug, Default)]
pub struct Named {
    pub id: String,
    pub label: Label,
    pub span: Span,
    pub generated: bool,
}
#[derive(Clone, Debug, Default)]
pub struct DraftElement {
    pub named: Named,
    pub kind: String,
    pub parent: Option<usize>,
    pub shape: Option<String>,
    pub metadata: Option<Vec<Property>>,
    pub stereotype: Option<String>,
    pub classes: Vec<String>,
}
#[derive(Clone, Debug, Default)]
pub struct DraftBoundary {
    pub named: Named,
    pub classes: Vec<String>,
}
#[derive(Clone, Debug, Default)]
pub struct DraftJson {
    pub id: String,
    pub value: OrderedJson,
    pub span: Span,
    pub classes: Vec<String>,
}
#[derive(Clone, Debug, Default)]
pub struct Endpoint {
    pub named: Named,
    pub declares: bool,
    pub classed: bool,
}
#[derive(Clone, Debug, Default)]
pub struct DraftRelationship {
    pub source: Endpoint,
    pub target: Endpoint,
    pub span: Span,
    pub id: Option<String>,
    pub id_span: Option<Span>,
    pub kind: String,
    pub arrow_type: u8,
    pub label: Option<Label>,
    pub minlen: f64,
}
#[derive(Clone, Debug, Default)]
pub struct DraftNote {
    pub target: String,
    pub span: Span,
    pub label: Label,
}
#[derive(Clone, Debug, Default)]
pub struct DraftMetadata {
    pub target: String,
    pub span: Span,
    pub properties: Vec<Property>,
    pub statement: usize,
}
#[derive(Clone, Debug, Default)]
pub struct ClassAssign {
    pub targets: Vec<(String, Span)>,
    pub classes: Vec<String>,
}
#[derive(Clone, Debug, Default)]
pub struct StyleAssign {
    pub target: String,
    pub span: Span,
    pub styles: Vec<String>,
}
#[derive(Clone, Debug, Default)]
pub struct Drafts {
    pub elements: Vec<DraftElement>,
    pub boundaries: Vec<DraftBoundary>,
    pub jsons: Vec<DraftJson>,
    pub relationships: Vec<DraftRelationship>,
    pub notes: Vec<DraftNote>,
    pub metadata: Vec<DraftMetadata>,
    pub classes: Vec<ClassAssign>,
    pub styles: Vec<StyleAssign>,
    pub class_defs: Vec<(Vec<String>, Vec<String>)>,
    pub direction: Option<String>,
    pub acc_title: String,
    pub acc_description: String,
}
