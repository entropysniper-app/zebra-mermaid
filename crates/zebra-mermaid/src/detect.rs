//! Diagram types and their detection. Ported from Pele `src/detect.ts`.

use std::fmt;
use std::str::FromStr;

use crate::util::chars::{is_space, is_word};

/// The 33 diagram types, named as Pele and Mermaid name them.
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

fn c4(text: &str, t: &str) -> bool {
    t.starts_with("C4Context")
        || text.contains("C4Container")
        || text.contains("C4Component")
        || text.contains("C4Dynamic")
        || text.contains("C4Deployment")
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
    // `/^\s*C4Context|C4Container|.../`: only the first alternative is anchored.
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
        // Mermaid's C4 detector is anchored only for its first alternative.
        assert_eq!(detect("flowchart TD\n  A[C4Container]"), Some(DiagramType::C4));
    }

    #[test]
    fn names_round_trip() {
        for t in DiagramType::ALL {
            assert_eq!(t.as_str().parse::<DiagramType>(), Ok(t));
        }
    }
}
