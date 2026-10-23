//! Graph layout. Ported from Pele `src/layout/`.
//!
//! [`layered`] lays out one flat graph in ranks, [`compound`] nests such layouts for groups
//! (subgraphs, clusters), and [`lanes`] lays out swimlanes, where every top-level group is a
//! band that runs the length of the diagram.

pub mod compound;
pub mod lanes;
pub mod layered;

pub mod components;
mod crossings;
