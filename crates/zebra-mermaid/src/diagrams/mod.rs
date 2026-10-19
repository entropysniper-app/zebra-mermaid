//! The diagram types and what they share: parse input, render context and output.
//!
//! Each type lives in its own module behind a Cargo feature of the same name. A type whose
//! feature is off is reported as `unsupported-diagram`.

pub mod common;

#[cfg(feature = "agentflow")]
pub mod agentflow;
#[cfg(feature = "architecture")]
pub mod architecture;
#[cfg(feature = "block")]
pub mod block;
#[cfg(feature = "c4")]
pub mod c4;
#[cfg(feature = "class")]
pub mod class;
#[cfg(feature = "cynefin")]
pub mod cynefin;
#[cfg(feature = "er")]
pub mod er;
#[cfg(feature = "eventmodeling")]
pub mod eventmodeling;
#[cfg(feature = "flowchart")]
pub mod flowchart;
#[cfg(feature = "gantt")]
pub mod gantt;
#[cfg(feature = "gitGraph")]
pub mod git_graph;
#[cfg(feature = "info")]
pub mod info;
#[cfg(feature = "ishikawa")]
pub mod ishikawa;
#[cfg(feature = "journey")]
pub mod journey;
#[cfg(feature = "kanban")]
pub mod kanban;
#[cfg(feature = "mindmap")]
pub mod mindmap;
#[cfg(feature = "packet")]
pub mod packet;
#[cfg(feature = "pie")]
pub mod pie;
#[cfg(feature = "quadrantChart")]
pub mod quadrant;
#[cfg(feature = "radar")]
pub mod radar;
#[cfg(feature = "railroad")]
pub mod railroad;
#[cfg(feature = "requirement")]
pub mod requirement;
#[cfg(feature = "sankey")]
pub mod sankey;
#[cfg(feature = "sequence")]
pub mod sequence;
#[cfg(feature = "state")]
pub mod state;
#[cfg(feature = "swimlane")]
pub mod swimlane;
#[cfg(feature = "timeline")]
pub mod timeline;
#[cfg(feature = "treemap")]
pub mod treemap;
#[cfg(feature = "treeView")]
pub mod treeview;
#[cfg(feature = "usecase")]
pub mod usecase;
#[cfg(feature = "venn")]
pub mod venn;
#[cfg(feature = "wardley")]
pub mod wardley;
#[cfg(feature = "xychart")]
pub mod xychart;

use crate::config::Config;
use crate::detect::DiagramType;
use crate::error::{Error, ErrorCode};
use crate::options::{ParseLimits, RenderOptions};
use crate::text::{Icons, Measure, TextMeasurer};
use crate::theme::{Role, Theming};
use crate::util::url::LinkInfo;

/// What a diagram parser is given. `source` has been through preprocessing and entity
/// encoding and ends with a newline, as Mermaid's parsers expect.
pub struct ParseInput<'a> {
    pub source: &'a str,
    pub config: &'a Config,
    pub title: Option<&'a str>,
    /// The number of front matter lines removed before `source`, for diagrams that report
    /// positions in the original text.
    pub line_offset: usize,
    pub limits: ParseLimits,
}

/// What a diagram's renderer is given besides its model and configuration.
pub struct RenderCtx<'a> {
    /// The host's options. `responsive` there is the host's; the settled value is
    /// [`RenderCtx::responsive`].
    pub options: &'a RenderOptions,
    /// Whether the SVG scales with its container: the host's choice, else true unless the
    /// diagram's configuration sets `useMaxWidth: false`.
    pub responsive: bool,
    measurer: &'a dyn TextMeasurer,
    icons: &'a Icons,
    /// The color choices of this render.
    pub theming: &'a Theming,
}

impl<'a> RenderCtx<'a> {
    pub fn new(
        options: &'a RenderOptions,
        responsive: bool,
        measurer: &'a dyn TextMeasurer,
        icons: &'a Icons,
        theming: &'a Theming,
    ) -> Self {
        RenderCtx { options, responsive, measurer, icons, theming }
    }

    /// The color of a part playing `role`, where Pele writes `css`. See [`Theming::paint`].
    #[inline]
    pub fn paint<'s>(&'s self, role: Role, css: &'s str) -> &'s str {
        self.theming.paint(role, css)
    }

    /// The fill of data series `index`.
    pub fn series(&self, index: i64) -> String {
        self.theming.series(index)
    }

    /// Whether colors are written as concrete values.
    pub fn resolved(&self) -> bool {
        self.theming.resolved_mode()
    }

    /// The measurer and icons for label layout.
    pub fn measure(&self) -> Measure<'a> {
        Measure::new(self.measurer, self.icons)
    }

    /// The host's icon resolver, when it gave one, as Pele passes `options.icons`.
    pub fn icons(&self) -> Option<&'a Icons> {
        self.icons.has_resolver().then_some(self.icons)
    }

    pub fn font_size(&self) -> f64 {
        self.options.font_size.unwrap_or(16.0)
    }

    pub fn padding(&self) -> f64 {
        self.options.padding.unwrap_or(8.0)
    }
}

/// A drawn diagram.
#[derive(Debug, Clone, PartialEq)]
pub struct Rendered {
    pub svg: String,
    pub width: f64,
    pub height: f64,
    pub links: Vec<LinkInfo>,
}

/// The parsed model of a diagram, by type.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "model", rename_all = "camelCase"))]
#[non_exhaustive]
pub enum DiagramModel {
    #[cfg(feature = "flowchart")]
    Flowchart(flowchart::FlowchartModel),
    #[cfg(feature = "sequence")]
    Sequence(sequence::SequenceModel),
    #[cfg(feature = "class")]
    Class(class::ClassModel),
    #[cfg(feature = "state")]
    State(state::StateModel),
    #[cfg(feature = "er")]
    Er(er::ErModel),
    #[cfg(feature = "pie")]
    Pie(pie::PieModel),
    #[cfg(feature = "gantt")]
    Gantt(gantt::GanttModel),
    #[cfg(feature = "gitGraph")]
    GitGraph(git_graph::GitModel),
    #[cfg(feature = "mindmap")]
    Mindmap(mindmap::MindmapModel),
    #[cfg(feature = "timeline")]
    Timeline(timeline::TimelineModel),
    #[cfg(feature = "journey")]
    Journey(journey::JourneyModel),
    #[cfg(feature = "xychart")]
    Xychart(xychart::XyChartModel),
    #[cfg(feature = "quadrantChart")]
    QuadrantChart(quadrant::QuadrantModel),
    #[cfg(feature = "requirement")]
    Requirement(requirement::RequirementModel),
    #[cfg(feature = "c4")]
    C4(c4::C4Model),
    #[cfg(feature = "sankey")]
    Sankey(sankey::SankeyModel),
    #[cfg(feature = "block")]
    Block(block::BlockModel),
    #[cfg(feature = "packet")]
    Packet(packet::PacketModel),
    #[cfg(feature = "kanban")]
    Kanban(kanban::KanbanModel),
    #[cfg(feature = "architecture")]
    Architecture(architecture::ArchitectureModel),
    #[cfg(feature = "radar")]
    Radar(radar::RadarModel),
    #[cfg(feature = "treemap")]
    Treemap(treemap::TreemapModel),
    #[cfg(feature = "info")]
    Info(info::InfoModel),
    #[cfg(feature = "agentflow")]
    Agentflow(agentflow::AgentflowModel),
    #[cfg(feature = "usecase")]
    Usecase(usecase::UsecaseModel),
    #[cfg(feature = "railroad")]
    Railroad(railroad::RailroadModel),
    #[cfg(feature = "treeView")]
    TreeView(treeview::TreeViewModel),
    #[cfg(feature = "venn")]
    Venn(venn::VennModel),
    #[cfg(feature = "cynefin")]
    Cynefin(cynefin::CynefinModel),
    #[cfg(feature = "wardley")]
    Wardley(wardley::WardleyModel),
    #[cfg(feature = "swimlane")]
    Swimlane(swimlane::SwimlaneModel),
    #[cfg(feature = "ishikawa")]
    Ishikawa(ishikawa::IshikawaModel),
    #[cfg(feature = "eventmodeling")]
    Eventmodeling(eventmodeling::EventModelingModel),
}

impl DiagramModel {
    pub fn diagram_type(&self) -> DiagramType {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "flowchart")]
            DiagramModel::Flowchart(_) => DiagramType::Flowchart,
            #[cfg(feature = "sequence")]
            DiagramModel::Sequence(_) => DiagramType::Sequence,
            #[cfg(feature = "class")]
            DiagramModel::Class(_) => DiagramType::Class,
            #[cfg(feature = "state")]
            DiagramModel::State(_) => DiagramType::State,
            #[cfg(feature = "er")]
            DiagramModel::Er(_) => DiagramType::Er,
            #[cfg(feature = "pie")]
            DiagramModel::Pie(_) => DiagramType::Pie,
            #[cfg(feature = "gantt")]
            DiagramModel::Gantt(_) => DiagramType::Gantt,
            #[cfg(feature = "gitGraph")]
            DiagramModel::GitGraph(_) => DiagramType::GitGraph,
            #[cfg(feature = "mindmap")]
            DiagramModel::Mindmap(_) => DiagramType::Mindmap,
            #[cfg(feature = "timeline")]
            DiagramModel::Timeline(_) => DiagramType::Timeline,
            #[cfg(feature = "journey")]
            DiagramModel::Journey(_) => DiagramType::Journey,
            #[cfg(feature = "xychart")]
            DiagramModel::Xychart(_) => DiagramType::Xychart,
            #[cfg(feature = "quadrantChart")]
            DiagramModel::QuadrantChart(_) => DiagramType::QuadrantChart,
            #[cfg(feature = "requirement")]
            DiagramModel::Requirement(_) => DiagramType::Requirement,
            #[cfg(feature = "c4")]
            DiagramModel::C4(_) => DiagramType::C4,
            #[cfg(feature = "sankey")]
            DiagramModel::Sankey(_) => DiagramType::Sankey,
            #[cfg(feature = "block")]
            DiagramModel::Block(_) => DiagramType::Block,
            #[cfg(feature = "packet")]
            DiagramModel::Packet(_) => DiagramType::Packet,
            #[cfg(feature = "kanban")]
            DiagramModel::Kanban(_) => DiagramType::Kanban,
            #[cfg(feature = "architecture")]
            DiagramModel::Architecture(_) => DiagramType::Architecture,
            #[cfg(feature = "radar")]
            DiagramModel::Radar(_) => DiagramType::Radar,
            #[cfg(feature = "treemap")]
            DiagramModel::Treemap(_) => DiagramType::Treemap,
            #[cfg(feature = "info")]
            DiagramModel::Info(_) => DiagramType::Info,
            #[cfg(feature = "agentflow")]
            DiagramModel::Agentflow(_) => DiagramType::Agentflow,
            #[cfg(feature = "usecase")]
            DiagramModel::Usecase(_) => DiagramType::Usecase,
            #[cfg(feature = "railroad")]
            DiagramModel::Railroad(_) => DiagramType::Railroad,
            #[cfg(feature = "treeView")]
            DiagramModel::TreeView(_) => DiagramType::TreeView,
            #[cfg(feature = "venn")]
            DiagramModel::Venn(_) => DiagramType::Venn,
            #[cfg(feature = "cynefin")]
            DiagramModel::Cynefin(_) => DiagramType::Cynefin,
            #[cfg(feature = "wardley")]
            DiagramModel::Wardley(_) => DiagramType::Wardley,
            #[cfg(feature = "swimlane")]
            DiagramModel::Swimlane(_) => DiagramType::Swimlane,
            #[cfg(feature = "ishikawa")]
            DiagramModel::Ishikawa(_) => DiagramType::Ishikawa,
            #[cfg(feature = "eventmodeling")]
            DiagramModel::Eventmodeling(_) => DiagramType::Eventmodeling,
        }
    }
}

/// Whether this build renders the type.
pub fn enabled(t: DiagramType) -> bool {
    match t {
        DiagramType::Flowchart => cfg!(feature = "flowchart"),
        DiagramType::Sequence => cfg!(feature = "sequence"),
        DiagramType::Class => cfg!(feature = "class"),
        DiagramType::State => cfg!(feature = "state"),
        DiagramType::Er => cfg!(feature = "er"),
        DiagramType::Pie => cfg!(feature = "pie"),
        DiagramType::Gantt => cfg!(feature = "gantt"),
        DiagramType::GitGraph => cfg!(feature = "gitGraph"),
        DiagramType::Mindmap => cfg!(feature = "mindmap"),
        DiagramType::Timeline => cfg!(feature = "timeline"),
        DiagramType::Journey => cfg!(feature = "journey"),
        DiagramType::Xychart => cfg!(feature = "xychart"),
        DiagramType::QuadrantChart => cfg!(feature = "quadrantChart"),
        DiagramType::Requirement => cfg!(feature = "requirement"),
        DiagramType::C4 => cfg!(feature = "c4"),
        DiagramType::Sankey => cfg!(feature = "sankey"),
        DiagramType::Block => cfg!(feature = "block"),
        DiagramType::Packet => cfg!(feature = "packet"),
        DiagramType::Kanban => cfg!(feature = "kanban"),
        DiagramType::Architecture => cfg!(feature = "architecture"),
        DiagramType::Radar => cfg!(feature = "radar"),
        DiagramType::Treemap => cfg!(feature = "treemap"),
        DiagramType::Info => cfg!(feature = "info"),
        DiagramType::Agentflow => cfg!(feature = "agentflow"),
        DiagramType::Usecase => cfg!(feature = "usecase"),
        DiagramType::Railroad => cfg!(feature = "railroad"),
        DiagramType::TreeView => cfg!(feature = "treeView"),
        DiagramType::Venn => cfg!(feature = "venn"),
        DiagramType::Cynefin => cfg!(feature = "cynefin"),
        DiagramType::Wardley => cfg!(feature = "wardley"),
        DiagramType::Swimlane => cfg!(feature = "swimlane"),
        DiagramType::Ishikawa => cfg!(feature = "ishikawa"),
        DiagramType::Eventmodeling => cfg!(feature = "eventmodeling"),
    }
}

pub(crate) fn unsupported(t: DiagramType) -> Error {
    Error::new(format!("Diagram type \"{}\" is not supported yet.", t), ErrorCode::UnsupportedDiagram).with_type(t)
}

#[allow(unused_variables)]
pub(crate) fn parse(t: DiagramType, input: &ParseInput) -> Result<DiagramModel, Error> {
    match t {
        #[cfg(feature = "flowchart")]
        DiagramType::Flowchart => flowchart::parse(input).map(DiagramModel::Flowchart),
        #[cfg(feature = "sequence")]
        DiagramType::Sequence => sequence::parse(input).map(DiagramModel::Sequence),
        #[cfg(feature = "class")]
        DiagramType::Class => class::parse(input).map(DiagramModel::Class),
        #[cfg(feature = "state")]
        DiagramType::State => state::parse(input).map(DiagramModel::State),
        #[cfg(feature = "er")]
        DiagramType::Er => er::parse(input).map(DiagramModel::Er),
        #[cfg(feature = "pie")]
        DiagramType::Pie => pie::parse(input).map(DiagramModel::Pie),
        #[cfg(feature = "gantt")]
        DiagramType::Gantt => gantt::parse(input).map(DiagramModel::Gantt),
        #[cfg(feature = "gitGraph")]
        DiagramType::GitGraph => git_graph::parse(input).map(DiagramModel::GitGraph),
        #[cfg(feature = "mindmap")]
        DiagramType::Mindmap => mindmap::parse(input).map(DiagramModel::Mindmap),
        #[cfg(feature = "timeline")]
        DiagramType::Timeline => timeline::parse(input).map(DiagramModel::Timeline),
        #[cfg(feature = "journey")]
        DiagramType::Journey => journey::parse(input).map(DiagramModel::Journey),
        #[cfg(feature = "xychart")]
        DiagramType::Xychart => xychart::parse(input).map(DiagramModel::Xychart),
        #[cfg(feature = "quadrantChart")]
        DiagramType::QuadrantChart => quadrant::parse(input).map(DiagramModel::QuadrantChart),
        #[cfg(feature = "requirement")]
        DiagramType::Requirement => requirement::parse(input).map(DiagramModel::Requirement),
        #[cfg(feature = "c4")]
        DiagramType::C4 => c4::parse(input).map(DiagramModel::C4),
        #[cfg(feature = "sankey")]
        DiagramType::Sankey => sankey::parse(input).map(DiagramModel::Sankey),
        #[cfg(feature = "block")]
        DiagramType::Block => block::parse(input).map(DiagramModel::Block),
        #[cfg(feature = "packet")]
        DiagramType::Packet => packet::parse(input).map(DiagramModel::Packet),
        #[cfg(feature = "kanban")]
        DiagramType::Kanban => kanban::parse(input).map(DiagramModel::Kanban),
        #[cfg(feature = "architecture")]
        DiagramType::Architecture => architecture::parse(input).map(DiagramModel::Architecture),
        #[cfg(feature = "radar")]
        DiagramType::Radar => radar::parse(input).map(DiagramModel::Radar),
        #[cfg(feature = "treemap")]
        DiagramType::Treemap => treemap::parse(input).map(DiagramModel::Treemap),
        #[cfg(feature = "info")]
        DiagramType::Info => info::parse(input).map(DiagramModel::Info),
        #[cfg(feature = "agentflow")]
        DiagramType::Agentflow => agentflow::parse(input).map(DiagramModel::Agentflow),
        #[cfg(feature = "usecase")]
        DiagramType::Usecase => usecase::parse(input).map(DiagramModel::Usecase),
        #[cfg(feature = "railroad")]
        DiagramType::Railroad => railroad::parse(input).map(DiagramModel::Railroad),
        #[cfg(feature = "treeView")]
        DiagramType::TreeView => treeview::parse(input).map(DiagramModel::TreeView),
        #[cfg(feature = "venn")]
        DiagramType::Venn => venn::parse(input).map(DiagramModel::Venn),
        #[cfg(feature = "cynefin")]
        DiagramType::Cynefin => cynefin::parse(input).map(DiagramModel::Cynefin),
        #[cfg(feature = "wardley")]
        DiagramType::Wardley => wardley::parse(input).map(DiagramModel::Wardley),
        #[cfg(feature = "swimlane")]
        DiagramType::Swimlane => swimlane::parse(input).map(DiagramModel::Swimlane),
        #[cfg(feature = "ishikawa")]
        DiagramType::Ishikawa => ishikawa::parse(input).map(DiagramModel::Ishikawa),
        #[cfg(feature = "eventmodeling")]
        DiagramType::Eventmodeling => eventmodeling::parse(input).map(DiagramModel::Eventmodeling),
        #[allow(unreachable_patterns)]
        _ => Err(unsupported(t)),
    }
}

#[allow(unused_variables)]
pub(crate) fn render(model: &DiagramModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    match model {
        #[cfg(feature = "flowchart")]
        DiagramModel::Flowchart(model) => flowchart::render(model, config, ctx),
        #[cfg(feature = "sequence")]
        DiagramModel::Sequence(model) => sequence::render(model, config, ctx),
        #[cfg(feature = "class")]
        DiagramModel::Class(model) => class::render(model, config, ctx),
        #[cfg(feature = "state")]
        DiagramModel::State(model) => state::render(model, config, ctx),
        #[cfg(feature = "er")]
        DiagramModel::Er(model) => er::render(model, config, ctx),
        #[cfg(feature = "pie")]
        DiagramModel::Pie(model) => pie::render(model, config, ctx),
        #[cfg(feature = "gantt")]
        DiagramModel::Gantt(model) => gantt::render(model, config, ctx),
        #[cfg(feature = "gitGraph")]
        DiagramModel::GitGraph(model) => git_graph::render(model, config, ctx),
        #[cfg(feature = "mindmap")]
        DiagramModel::Mindmap(model) => mindmap::render(model, config, ctx),
        #[cfg(feature = "timeline")]
        DiagramModel::Timeline(model) => timeline::render(model, config, ctx),
        #[cfg(feature = "journey")]
        DiagramModel::Journey(model) => journey::render(model, config, ctx),
        #[cfg(feature = "xychart")]
        DiagramModel::Xychart(model) => xychart::render(model, config, ctx),
        #[cfg(feature = "quadrantChart")]
        DiagramModel::QuadrantChart(model) => quadrant::render(model, config, ctx),
        #[cfg(feature = "requirement")]
        DiagramModel::Requirement(model) => requirement::render(model, config, ctx),
        #[cfg(feature = "c4")]
        DiagramModel::C4(model) => c4::render(model, config, ctx),
        #[cfg(feature = "sankey")]
        DiagramModel::Sankey(model) => sankey::render(model, config, ctx),
        #[cfg(feature = "block")]
        DiagramModel::Block(model) => block::render(model, config, ctx),
        #[cfg(feature = "packet")]
        DiagramModel::Packet(model) => packet::render(model, config, ctx),
        #[cfg(feature = "kanban")]
        DiagramModel::Kanban(model) => kanban::render(model, config, ctx),
        #[cfg(feature = "architecture")]
        DiagramModel::Architecture(model) => architecture::render(model, config, ctx),
        #[cfg(feature = "radar")]
        DiagramModel::Radar(model) => radar::render(model, config, ctx),
        #[cfg(feature = "treemap")]
        DiagramModel::Treemap(model) => treemap::render(model, config, ctx),
        #[cfg(feature = "info")]
        DiagramModel::Info(model) => info::render(model, config, ctx),
        #[cfg(feature = "agentflow")]
        DiagramModel::Agentflow(model) => agentflow::render(model, config, ctx),
        #[cfg(feature = "usecase")]
        DiagramModel::Usecase(model) => usecase::render(model, config, ctx),
        #[cfg(feature = "railroad")]
        DiagramModel::Railroad(model) => railroad::render(model, config, ctx),
        #[cfg(feature = "treeView")]
        DiagramModel::TreeView(model) => treeview::render(model, config, ctx),
        #[cfg(feature = "venn")]
        DiagramModel::Venn(model) => venn::render(model, config, ctx),
        #[cfg(feature = "cynefin")]
        DiagramModel::Cynefin(model) => cynefin::render(model, config, ctx),
        #[cfg(feature = "wardley")]
        DiagramModel::Wardley(model) => wardley::render(model, config, ctx),
        #[cfg(feature = "swimlane")]
        DiagramModel::Swimlane(model) => swimlane::render(model, config, ctx),
        #[cfg(feature = "ishikawa")]
        DiagramModel::Ishikawa(model) => ishikawa::render(model, config, ctx),
        #[cfg(feature = "eventmodeling")]
        DiagramModel::Eventmodeling(model) => eventmodeling::render(model, config, ctx),
    }
}

/// `.replace(/\n\s+/g, '\n')`: a line break and the white space after it become one break.
pub(crate) fn squeeze_line_starts(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        out.push(c);
        if c == '\n' {
            while chars.peek().is_some_and(|&n| crate::util::chars::is_space(n)) {
                chars.next();
            }
        }
    }
    out
}
