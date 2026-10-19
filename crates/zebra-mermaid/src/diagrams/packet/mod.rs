//! Packet diagrams. Ported from Pele `src/diagrams/packet/`.

mod parser;
mod render;

use crate::config::{Config, ConfigValue};
use crate::detect::DiagramType;
use crate::diagrams::ParseInput;
use crate::error::Error;
use crate::util::js;

pub use parser::{parse_packet, PacketAst, PacketBlockAst};
pub(crate) use render::render;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PacketBlock {
    pub start: f64,
    pub end: f64,
    /// As Mermaid counts it: one less than the width for the pieces of a field split across rows.
    pub bits: f64,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PacketModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    pub bits_per_row: f64,
    pub rows: Vec<Vec<PacketBlock>>,
}

const MAX_ROWS: usize = 10_000;

/// A number from the `packet` section of the configuration, at least `min` and capped at `max`.
pub(crate) fn packet_option(config: &Config, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match config.lookup("packet", key).and_then(ConfigValue::as_f64) {
        Some(value) if value >= min => js::min(value, max),
        _ => fallback,
    }
}

fn invalid(message: String) -> Error {
    Error::semantic(message).with_type(DiagramType::Packet)
}

pub fn build_packet(ast: PacketAst, config: &Config, title: Option<&str>) -> Result<PacketModel, Error> {
    let bits_per_row = packet_option(config, "bitsPerRow", 32.0, 1.0, 4096.0).floor();
    let mut rows: Vec<Vec<PacketBlock>> = Vec::new();
    let mut last_bit = -1.0;
    let mut word: Vec<PacketBlock> = Vec::new();
    let mut row = 1.0;
    let s = js::to_string;

    for block in &ast.blocks {
        let (start, end, bits) = (block.start, block.end, block.bits);
        if let (Some(start), Some(end)) = (start, end) {
            if end < start {
                return Err(invalid(format!("Packet block {} - {} is invalid. End must be greater than start.", s(start), s(end))));
            }
        }
        let mut start = start.unwrap_or(last_bit + 1.0);
        if start != last_bit + 1.0 {
            return Err(invalid(format!(
                "Packet block {} - {} is not contiguous. It should start from {}.",
                s(start),
                s(end.unwrap_or(start)),
                s(last_bit + 1.0)
            )));
        }
        if bits == Some(0.0) {
            return Err(invalid(format!("Packet block {} is invalid. Cannot have a zero bit field.", s(start))));
        }
        let end = end.unwrap_or(start + bits.unwrap_or(1.0) - 1.0);
        let mut bits = bits.unwrap_or(end - start + 1.0);
        last_bit = end;

        while word.len() as f64 <= bits_per_row + 1.0 && rows.len() < MAX_ROWS {
            let row_end = row * bits_per_row - 1.0;
            if end <= row_end {
                word.push(PacketBlock { start, end, bits, label: block.label.clone() });
                if end == row_end {
                    rows.push(std::mem::take(&mut word));
                    row += 1.0;
                }
                break;
            }
            word.push(PacketBlock { start, end: row_end, bits: row_end - start, label: block.label.clone() });
            rows.push(std::mem::take(&mut word));
            row += 1.0;
            start = row_end + 1.0;
            bits = end - start;
        }
    }
    if !word.is_empty() {
        rows.push(word);
    }

    Ok(PacketModel {
        title: ast.title.filter(|t| !t.is_empty()).or_else(|| title.map(str::to_string)),
        acc_title: ast.acc_title.filter(|t| !t.is_empty()).map(|t| js::trim_start(&t).to_string()),
        acc_descr: ast.acc_descr.filter(|t| !t.is_empty()).map(|t| crate::diagrams::squeeze_line_starts(&t)),
        bits_per_row,
        rows,
    })
}

pub(crate) fn parse(input: &ParseInput) -> Result<PacketModel, Error> {
    build_packet(parse_packet(input.source)?, input.config, input.title)
}
