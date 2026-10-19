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

#[path = "upstream/compound.rs"]
mod compound;

#[path = "upstream/lanes.rs"]
mod lanes;

#[path = "upstream/utilities.rs"]
mod utilities;
