//! Direct ports of individual tests from the pinned Pele suite.
//! tests/upstream/ports.json maps every function to one collected upstream test.
#[path = "upstream/edges.rs"]
mod edges;
#[cfg(feature = "info")]
#[path = "upstream/info_parser.rs"]
mod info_parser;
#[cfg(feature = "info")]
#[path = "upstream/info_render.rs"]
mod info_render;
#[cfg(feature = "info")]
#[path = "upstream/info_stress.rs"]
mod info_stress;

#[cfg(all(feature = "flowchart", feature = "class", feature = "kanban", feature = "treeView", feature = "sequence", feature = "timeline", feature = "pie"))]
#[path = "upstream/fonts_render.rs"]
mod fonts_render;

#[cfg(all(feature = "flowchart", feature = "class", feature = "state", feature = "railroad", feature = "sequence", feature = "pie"))]
#[path = "upstream/mermaid_type.rs"]
mod mermaid_type;

#[cfg(feature = "flowchart")]
#[path = "upstream/cjk_wrap.rs"]
mod cjk_wrap;

#[path = "upstream/compound.rs"]
mod compound;

#[path = "upstream/lanes.rs"]
mod lanes;

#[path = "upstream/utilities.rs"]
mod utilities;
