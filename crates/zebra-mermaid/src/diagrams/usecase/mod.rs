//! Use-case diagrams, including source occurrences and ordered JSON tables.
mod db;
pub mod graph;
mod json;
mod lexer;
mod parser;
mod render;
mod tokens;
mod types;
use super::ParseInput;
use crate::Error;
pub(crate) use render::render;
pub use types::UsecaseModel;
pub(crate) fn parse(input: &ParseInput) -> Result<UsecaseModel, Error> {
    parser::parse(input.source, input.title)
}
