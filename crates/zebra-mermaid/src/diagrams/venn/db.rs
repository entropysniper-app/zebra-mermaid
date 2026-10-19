//! The model a venn diagram is read into. Ported from Pele `src/diagrams/venn/db.ts`.

use std::collections::HashSet;

use indexmap::IndexMap;

use super::{VennModel, VennStyle, VennSubset, VennText};
use crate::detect::DiagramType;
use crate::error::Error;
use crate::util::{js, math};

/// Mermaid takes one pair of quotes off an id, label or style value.
pub fn normalize_text(text: &str) -> String {
    let trimmed = js::trim(text);
    if js::len16(trimmed) >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

/// `ids.map(normalizeText).sort()`: the default sort compares UTF-16 code units.
fn normalize_ids(ids: &[String]) -> Vec<String> {
    let mut sets: Vec<String> = ids.iter().map(|id| normalize_text(id)).collect();
    sets.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
    sets
}

pub(crate) struct VennDb<'a> {
    pub model: &'a mut VennModel,
    known: HashSet<String>,
    current: Option<Vec<String>>,
    // Whether an indented `text` line belongs to the set or union before it. The lexer reads this.
    indent: bool,
}

impl<'a> VennDb<'a> {
    pub fn new(model: &'a mut VennModel) -> Self {
        VennDb { model, known: HashSet::new(), current: None, indent: false }
    }

    pub fn set_diagram_title(&mut self, title: String) {
        self.model.title = Some(title);
    }

    /// A set without a size gets 10, and an overlap of n sets 10 / n².
    pub fn add_subset_data(&mut self, ids: &[String], label: Option<&str>, size: Option<f64>) {
        let sets = normalize_ids(ids);
        self.current = Some(sets.clone());
        if sets.len() == 1 {
            self.known.insert(sets[0].clone());
        }
        let size = size.unwrap_or_else(|| 10.0 / math::pow(ids.len() as f64, 2.0));
        let label = label.filter(|l| !l.is_empty()).map(normalize_text);
        self.model.subsets.push(VennSubset { sets, size, label });
    }

    pub fn add_text_data(&mut self, ids: &[String], id: &str, label: Option<&str>) {
        let label = label.filter(|l| !l.is_empty()).map(normalize_text);
        self.model.text_nodes.push(VennText { sets: normalize_ids(ids), id: normalize_text(id), label });
    }

    pub fn add_style_data(&mut self, ids: &[String], data: Vec<(String, String)>) {
        let mut styles = IndexMap::new();
        for (key, value) in data {
            let value = if value.is_empty() { value } else { normalize_text(&value) };
            styles.insert(key, value);
        }
        self.model.styles.push(VennStyle { targets: normalize_ids(ids), styles });
    }

    pub fn validate_union_identifiers(&self, ids: &[String]) -> Result<(), Error> {
        let unknown: Vec<String> = ids.iter().map(|id| normalize_text(id)).filter(|id| !self.known.contains(id)).collect();
        if !unknown.is_empty() {
            return Err(Error::semantic(format!("unknown set identifier: {}", unknown.join(", "))).with_type(DiagramType::Venn));
        }
        Ok(())
    }

    pub fn current_sets(&self) -> Option<&Vec<String>> {
        self.current.as_ref()
    }

    pub fn indent_mode(&self) -> bool {
        self.indent
    }

    pub fn set_indent_mode(&mut self, enabled: bool) {
        self.indent = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_like_mermaid() {
        assert_eq!(normalize_text("  \"a b\" "), "a b");
        assert_eq!(normalize_text("\""), "\"");
        assert_eq!(normalize_text("\"\""), "");
        assert_eq!(normalize_text("x"), "x");
    }

    #[test]
    fn sorts_in_code_units() {
        // U+FF21 sorts before U+1F600 by code point, after it by UTF-16 code unit.
        let ids = vec!["\u{1F600}".to_string(), "\u{FF21}".to_string(), "B".to_string()];
        assert_eq!(normalize_ids(&ids), vec!["B", "\u{1F600}", "\u{FF21}"]);
    }
}
