//! Diagram types and their detection. Ported from Pele `src/detect.ts`.

use std::fmt;
use std::str::FromStr;

use crate::util::chars::{is_space, is_word};

/// The 33 diagram types, named as Pele names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiagramType {
    Flowchart,
    Sequence,
    Class,
    State,
    Er,
    Pie,
    Gantt,
    GitGraph,
    Mindmap,
    Timeline,
    Journey,
    Xychart,
    QuadrantChart,
    Requirement,
    C4,
    Sankey,
    Block,
    Packet,
    Kanban,
    Architecture,
    Radar,
    Treemap,
    Info,
    Agentflow,
    Usecase,
    Railroad,
    TreeView,
    Venn,
    Cynefin,
    Wardley,
    Swimlane,
    Ishikawa,
    Eventmodeling,
}

impl DiagramType {
    pub const ALL: [DiagramType; 33] = [
        DiagramType::Flowchart,
        DiagramType::Sequence,
        DiagramType::Class,
        DiagramType::State,
        DiagramType::Er,
        DiagramType::Pie,
        DiagramType::Gantt,
        DiagramType::GitGraph,
        DiagramType::Mindmap,
        DiagramType::Timeline,
        DiagramType::Journey,
        DiagramType::Xychart,
        DiagramType::QuadrantChart,
        DiagramType::Requirement,
        DiagramType::C4,
        DiagramType::Sankey,
        DiagramType::Block,
        DiagramType::Packet,
        DiagramType::Kanban,
        DiagramType::Architecture,
        DiagramType::Radar,
        DiagramType::Treemap,
        DiagramType::Info,
        DiagramType::Agentflow,
        DiagramType::Usecase,
        DiagramType::Railroad,
        DiagramType::TreeView,
        DiagramType::Venn,
        DiagramType::Cynefin,
        DiagramType::Wardley,
        DiagramType::Swimlane,
        DiagramType::Ishikawa,
        DiagramType::Eventmodeling,
    ];

    /// The type's id, as Pele writes it in `type` and in the SVG class.
    pub fn as_str(self) -> &'static str {
        match self {
            DiagramType::Flowchart => "flowchart",
            DiagramType::Sequence => "sequence",
            DiagramType::Class => "class",
            DiagramType::State => "state",
            DiagramType::Er => "er",
            DiagramType::Pie => "pie",
            DiagramType::Gantt => "gantt",
            DiagramType::GitGraph => "gitGraph",
            DiagramType::Mindmap => "mindmap",
            DiagramType::Timeline => "timeline",
            DiagramType::Journey => "journey",
            DiagramType::Xychart => "xychart",
            DiagramType::QuadrantChart => "quadrantChart",
            DiagramType::Requirement => "requirement",
            DiagramType::C4 => "c4",
            DiagramType::Sankey => "sankey",
            DiagramType::Block => "block",
            DiagramType::Packet => "packet",
            DiagramType::Kanban => "kanban",
            DiagramType::Architecture => "architecture",
            DiagramType::Radar => "radar",
            DiagramType::Treemap => "treemap",
            DiagramType::Info => "info",
            DiagramType::Agentflow => "agentflow",
            DiagramType::Usecase => "usecase",
            DiagramType::Railroad => "railroad",
            DiagramType::TreeView => "treeView",
            DiagramType::Venn => "venn",
            DiagramType::Cynefin => "cynefin",
            DiagramType::Wardley => "wardley",
            DiagramType::Swimlane => "swimlane",
            DiagramType::Ishikawa => "ishikawa",
            DiagramType::Eventmodeling => "eventmodeling",
        }
    }

    /// The name of the type's section in Mermaid's configuration.
    pub fn config_section(self) -> &'static str {
        match self {
            DiagramType::Wardley => "wardley-beta",
            DiagramType::Xychart => "xyChart",
            other => other.as_str(),
        }
    }

    /// Whether the parser reads the text with its `%%` comment lines kept, to report positions.
    pub(crate) fn keeps_comments(self) -> bool {
        matches!(self, DiagramType::Agentflow | DiagramType::Eventmodeling)
    }
}

impl fmt::Display for DiagramType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DiagramType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        DiagramType::ALL.iter().copied().find(|t| t.as_str() == s).ok_or(())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DiagramType {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

/// The type name reported by Mermaid 12.1, preserved independently of Pele's SVG type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MermaidType {
    FlowchartV2,
    Sequence,
    ClassDiagram,
    StateDiagram,
    Er,
    Pie,
    Gantt,
    GitGraph,
    Mindmap,
    Timeline,
    Journey,
    Xychart,
    QuadrantChart,
    Requirement,
    C4,
    Sankey,
    Block,
    Packet,
    Kanban,
    Architecture,
    Radar,
    Treemap,
    Info,
    Agentflow,
    Usecase,
    Railroad,
    TreeView,
    Venn,
    Cynefin,
    Wardley,
    Swimlane,
    Ishikawa,
    Eventmodeling,
    FlowchartElk,
    RailroadEbnf,
    RailroadAbnf,
    RailroadPeg,
}

impl MermaidType {
    pub const ALL: [Self; 37] = [
        Self::FlowchartV2,
        Self::Sequence,
        Self::ClassDiagram,
        Self::StateDiagram,
        Self::Er,
        Self::Pie,
        Self::Gantt,
        Self::GitGraph,
        Self::Mindmap,
        Self::Timeline,
        Self::Journey,
        Self::Xychart,
        Self::QuadrantChart,
        Self::Requirement,
        Self::C4,
        Self::Sankey,
        Self::Block,
        Self::Packet,
        Self::Kanban,
        Self::Architecture,
        Self::Radar,
        Self::Treemap,
        Self::Info,
        Self::Agentflow,
        Self::Usecase,
        Self::Railroad,
        Self::TreeView,
        Self::Venn,
        Self::Cynefin,
        Self::Wardley,
        Self::Swimlane,
        Self::Ishikawa,
        Self::Eventmodeling,
        Self::FlowchartElk,
        Self::RailroadEbnf,
        Self::RailroadAbnf,
        Self::RailroadPeg,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::FlowchartV2 => "flowchart-v2",
            Self::Sequence => "sequence",
            Self::ClassDiagram => "classDiagram",
            Self::StateDiagram => "stateDiagram",
            Self::Er => "er",
            Self::Pie => "pie",
            Self::Gantt => "gantt",
            Self::GitGraph => "gitGraph",
            Self::Mindmap => "mindmap",
            Self::Timeline => "timeline",
            Self::Journey => "journey",
            Self::Xychart => "xychart",
            Self::QuadrantChart => "quadrantChart",
            Self::Requirement => "requirement",
            Self::C4 => "c4",
            Self::Sankey => "sankey",
            Self::Block => "block",
            Self::Packet => "packet",
            Self::Kanban => "kanban",
            Self::Architecture => "architecture",
            Self::Radar => "radar",
            Self::Treemap => "treemap",
            Self::Info => "info",
            Self::Agentflow => "agentflow",
            Self::Usecase => "usecase",
            Self::Railroad => "railroad",
            Self::TreeView => "treeView",
            Self::Venn => "venn",
            Self::Cynefin => "cynefin",
            Self::Wardley => "wardley",
            Self::Swimlane => "swimlane",
            Self::Ishikawa => "ishikawa",
            Self::Eventmodeling => "eventmodeling",
            Self::FlowchartElk => "flowchart-elk",
            Self::RailroadEbnf => "railroadEbnf",
            Self::RailroadAbnf => "railroadAbnf",
            Self::RailroadPeg => "railroadPeg",
        }
    }

    fn for_diagram(diagram_type: DiagramType) -> Self {
        match diagram_type {
            DiagramType::Flowchart => Self::FlowchartV2,
            DiagramType::Sequence => Self::Sequence,
            DiagramType::Class => Self::ClassDiagram,
            DiagramType::State => Self::StateDiagram,
            DiagramType::Er => Self::Er,
            DiagramType::Pie => Self::Pie,
            DiagramType::Gantt => Self::Gantt,
            DiagramType::GitGraph => Self::GitGraph,
            DiagramType::Mindmap => Self::Mindmap,
            DiagramType::Timeline => Self::Timeline,
            DiagramType::Journey => Self::Journey,
            DiagramType::Xychart => Self::Xychart,
            DiagramType::QuadrantChart => Self::QuadrantChart,
            DiagramType::Requirement => Self::Requirement,
            DiagramType::C4 => Self::C4,
            DiagramType::Sankey => Self::Sankey,
            DiagramType::Block => Self::Block,
            DiagramType::Packet => Self::Packet,
            DiagramType::Kanban => Self::Kanban,
            DiagramType::Architecture => Self::Architecture,
            DiagramType::Radar => Self::Radar,
            DiagramType::Treemap => Self::Treemap,
            DiagramType::Info => Self::Info,
            DiagramType::Agentflow => Self::Agentflow,
            DiagramType::Usecase => Self::Usecase,
            DiagramType::Railroad => Self::Railroad,
            DiagramType::TreeView => Self::TreeView,
            DiagramType::Venn => Self::Venn,
            DiagramType::Cynefin => Self::Cynefin,
            DiagramType::Wardley => Self::Wardley,
            DiagramType::Swimlane => Self::Swimlane,
            DiagramType::Ishikawa => Self::Ishikawa,
            DiagramType::Eventmodeling => Self::Eventmodeling,
        }
    }
}

impl fmt::Display for MermaidType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for MermaidType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Self::ALL.iter().copied().find(|t| t.as_str() == s).ok_or(())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for MermaidType {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

/// The two names selected by the first matching upstream detector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Detected {
    pub diagram_type: DiagramType,
    pub mermaid_type: MermaidType,
}

/// Detects both names in already preprocessed text, preserving Mermaid's detector order.
pub fn detect_diagram(text: &str) -> Option<Detected> {
    let diagram_type = detect(text)?;
    let t = after_space(text);
    let mermaid_type = match diagram_type {
        DiagramType::Flowchart if t.starts_with("flowchart-elk") => MermaidType::FlowchartElk,
        DiagramType::Railroad if starts_ci(t, "railroad-ebnf-beta") => MermaidType::RailroadEbnf,
        DiagramType::Railroad if starts_ci(t, "railroad-abnf-beta") => MermaidType::RailroadAbnf,
        DiagramType::Railroad if starts_ci(t, "railroad-peg-beta") => MermaidType::RailroadPeg,
        other => MermaidType::for_diagram(other),
    };
    Some(Detected { diagram_type, mermaid_type })
}

/// `^\s*`: the text after leading white space.
fn after_space(text: &str) -> &str {
    text.trim_start_matches(is_space)
}

fn starts_ci(s: &str, word: &str) -> bool {
    s.len() >= word.len() && s.as_bytes()[..word.len()].eq_ignore_ascii_case(word.as_bytes())
}

/// `\b` after a word character at the end of `prefix`: the next character is not `\w`.
fn boundary_after(rest: &str) -> bool {
    !rest.chars().next().is_some_and(is_word)
}

fn ishikawa(t: &str) -> bool {
    if !starts_ci(t, "ishikawa") {
        return false;
    }
    let rest = &t[8..];
    if starts_ci(rest, "-beta") && boundary_after(&rest[5..]) {
        return true;
    }
    boundary_after(rest)
}

fn railroad(t: &str) -> bool {
    if !starts_ci(t, "railroad-") {
        return false;
    }
    let rest = &t[9..];
    for variant in ["ebnf-", "abnf-", "peg-"] {
        if starts_ci(rest, variant) && starts_ci(&rest[variant.len()..], "beta") {
            return true;
        }
    }
    starts_ci(rest, "beta")
}

fn c4(_text: &str, t: &str) -> bool {
    t.starts_with("C4Context")
        || t.starts_with("C4Container")
        || t.starts_with("C4Component")
        || t.starts_with("C4Dynamic")
        || t.starts_with("C4Deployment")
}

/// Mermaid's detectors, in the order Mermaid registers them. The first match wins. `text` is
/// the preprocessed text, with front matter, directives and comment lines removed.
pub fn detect(text: &str) -> Option<DiagramType> {
    use DiagramType::*;
    let t = after_space(text);
    let word = |w: &str| t.starts_with(w);
    let word_b = |w: &str| t.starts_with(w) && boundary_after(&t[w.len()..]);
    if word("flowchart-elk") {
        return Some(Flowchart);
    }
    if word("mindmap") {
        return Some(Mindmap);
    }
    if word("architecture") {
        return Some(Architecture);
    }
    if word_b("agentflow-beta") {
        return Some(Agentflow);
    }
    // All C4 alternatives are anchored after leading white space.
    if c4(text, t) {
        return Some(C4);
    }
    if word("kanban") {
        return Some(Kanban);
    }
    if word("classDiagram") {
        return Some(Class);
    }
    if word("erDiagram") {
        return Some(Er);
    }
    if word("gantt") {
        return Some(Gantt);
    }
    if word("info") {
        return Some(Info);
    }
    if word("pie") {
        return Some(Pie);
    }
    if word("requirement") {
        return Some(Requirement);
    }
    if word("sequenceDiagram") {
        return Some(Sequence);
    }
    if word_b("swimlane-beta") {
        return Some(Swimlane);
    }
    if word("graph") || word("flowchart") {
        return Some(Flowchart);
    }
    if word("timeline") {
        return Some(Timeline);
    }
    if word("gitGraph") {
        return Some(GitGraph);
    }
    if word("stateDiagram") {
        return Some(State);
    }
    if word("journey") {
        return Some(Journey);
    }
    if word("quadrantChart") {
        return Some(QuadrantChart);
    }
    if word("sankey") {
        return Some(Sankey);
    }
    if word("packet") {
        return Some(Packet);
    }
    if word("xychart") {
        return Some(Xychart);
    }
    if word("block") {
        return Some(Block);
    }
    if word("eventmodeling") {
        return Some(Eventmodeling);
    }
    if word("treeView-beta") {
        return Some(TreeView);
    }
    if word("radar-beta") {
        return Some(Radar);
    }
    if ishikawa(t) {
        return Some(Ishikawa);
    }
    if word("treemap") {
        return Some(Treemap);
    }
    if railroad(t) {
        return Some(Railroad);
    }
    if word("venn-beta") {
        return Some(Venn);
    }
    if starts_ci(t, "wardley-beta") {
        return Some(Wardley);
    }
    if let Some(rest) = t.strip_prefix("cynefin-beta") {
        if rest.is_empty() || rest.starts_with(':') || rest.chars().next().is_some_and(is_space) {
            return Some(Cynefin);
        }
    }
    if let Some(rest) = t.strip_prefix("usecase-beta") {
        if rest.is_empty() || rest.chars().next().is_some_and(is_space) {
            return Some(Usecase);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_in_mermaid_order() {
        assert_eq!(detect("  pie title x"), Some(DiagramType::Pie));
        assert_eq!(detect("graph TD"), Some(DiagramType::Flowchart));
        assert_eq!(detect("flowchart-elk TD"), Some(DiagramType::Flowchart));
        assert_eq!(detect("\u{3000}info"), Some(DiagramType::Info));
        assert_eq!(detect("sequenceDiagram"), Some(DiagramType::Sequence));
        assert_eq!(detect("stateDiagram-v2"), Some(DiagramType::State));
        assert_eq!(detect("ISHIKAWA-Beta"), Some(DiagramType::Ishikawa));
        assert_eq!(detect("ishikawa-betax"), Some(DiagramType::Ishikawa));
        assert_eq!(detect("ishikawax"), None);
        assert_eq!(detect("Railroad-PEG-beta"), Some(DiagramType::Railroad));
        assert_eq!(detect("cynefin-beta:"), Some(DiagramType::Cynefin));
        assert_eq!(detect("cynefin-betax"), None);
        assert_eq!(detect("usecase-beta"), Some(DiagramType::Usecase));
        assert_eq!(detect("agentflow-beta2"), None);
        assert_eq!(detect("unknown"), None);
        // Pele 1.9.0 anchors every C4 alternative.
        assert_eq!(detect("flowchart TD\n  A[C4Container]"), Some(DiagramType::Flowchart));
    }

    #[test]
    fn names_round_trip() {
        for t in DiagramType::ALL {
            assert_eq!(t.as_str().parse::<DiagramType>(), Ok(t));
        }
    }
}
