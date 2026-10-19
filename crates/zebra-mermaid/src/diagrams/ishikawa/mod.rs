//! Ishikawa (fishbone) diagrams. Ported from Pele `src/diagrams/ishikawa/`.

mod lexer;
mod render;

use crate::detect::DiagramType;
use crate::diagrams::ParseInput;
use crate::error::{syntax_error, Error};
use crate::util::js;

pub(crate) use render::render;

use lexer::{tokenize, END, EOF, ISHIKAWA, NL, SPACELINE, SPACELIST, TEXT, TOKEN_NAMES};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IshikawaNode {
    pub text: String,
    /// Indices into [`IshikawaModel::nodes`].
    pub children: Vec<usize>,
}

/// The root is the effect; its children are the categories of causes, theirs the causes, and
/// so on. The outline may be nested as deep as its indentation goes, so the nodes are kept in a
/// flat list, the root first, and refer to their children by index: nothing that walks or drops
/// the tree recurses.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct IshikawaModel {
    pub title: Option<String>,
    /// Every node, in the order given; the first is the root, when there is one.
    pub nodes: Vec<IshikawaNode>,
}

impl IshikawaModel {
    /// The effect.
    pub fn root(&self) -> Option<&IshikawaNode> {
        self.nodes.first()
    }
}

/// Pele's `IshikawaDb`.
#[derive(Default)]
struct IshikawaDb {
    model: IshikawaModel,
    /// The line of ancestors of the last node, each with its level.
    stack: Vec<(i64, usize)>,
    base_level: Option<i64>,
}

impl IshikawaDb {
    /// The first line is the effect. Every later line is a cause, nested by its indentation
    /// relative to the first cause, so the effect itself may be indented more than the causes.
    fn add_node(&mut self, raw_level: i64, text: String) {
        let index = self.model.nodes.len();
        self.model.nodes.push(IshikawaNode { text, children: Vec::new() });
        if index == 0 {
            self.stack = vec![(0, index)];
            return;
        }
        let base = *self.base_level.get_or_insert(raw_level);
        let level = (raw_level - base + 1).max(1);
        while self.stack.len() > 1 && self.stack[self.stack.len() - 1].0 >= level {
            self.stack.pop();
        }
        let parent = self.stack[self.stack.len() - 1].1;
        self.model.nodes[parent].children.push(index);
        self.stack.push((level, index));
    }
}

const STOP: &str = "'SPACELINE', 'NL', 'EOF'";

/// Follows the parser Jison builds from Mermaid's grammar: blank lines and comments may come
/// first, then the keyword and at most one line break, then statements. A statement is a line
/// of text, with or without indentation, or a blank line; each is ended by a blank line, a line
/// break or the end of the text, and any number of line breaks after that.
pub fn parse_ishikawa(src: &str, title: Option<&str>) -> Result<IshikawaModel, Error> {
    let t = tokenize(src);
    let kind = |i: usize| t.types.get(i).copied();
    let fail = |i: usize, expected: &str| -> Error {
        let i = i.min(t.types.len() - 1);
        syntax_error(DiagramType::Ishikawa, src, t.starts[i], &format!("Expecting {expected}, got '{}'", TOKEN_NAMES[t.types[i]]), false)
    };
    let mut db = IshikawaDb::default();
    db.model.title = title.map(str::to_string);
    let mut i = 0;

    if kind(i) == Some(SPACELINE) {
        i += 1;
        while matches!(kind(i), Some(SPACELINE | NL)) {
            i += 1;
        }
        if kind(i) != Some(ISHIKAWA) {
            return Err(fail(i, "'SPACELINE', 'NL', 'ISHIKAWA'"));
        }
    } else if kind(i) != Some(ISHIKAWA) {
        return Err(fail(i, "'SPACELINE', 'ISHIKAWA'"));
    }
    i += 1;
    if kind(i) == Some(NL) {
        i += 1;
    }

    let ends_statement = |k: Option<usize>| matches!(k, Some(SPACELINE | NL | EOF));
    loop {
        let here = kind(i);
        if here == Some(TEXT) || (here == Some(SPACELIST) && kind(i + 1) == Some(TEXT)) {
            let text = if here == Some(TEXT) { i } else { i + 1 };
            i = text + 1;
            if !ends_statement(kind(i)) {
                return Err(fail(i, STOP));
            }
            // Indentation counts UTF-16 units, as Pele subtracts offsets.
            let level = if here == Some(TEXT) { 0 } else { js::len16(&src[t.starts[text - 1]..t.ends[text - 1]]) as i64 };
            db.add_node(level, js::trim(&src[t.starts[text]..t.ends[text]]).to_string());
        } else if here == Some(SPACELINE) || here == Some(SPACELIST) {
            i += 1;
            if !ends_statement(kind(i)) {
                return Err(fail(i, if here == Some(SPACELIST) { "'SPACELINE', 'NL', 'EOF', 'TEXT'" } else { STOP }));
            }
        } else {
            return Err(fail(i, "'SPACELINE', 'SPACELIST', 'TEXT'"));
        }
        i += 1;
        while matches!(kind(i), Some(NL | EOF)) {
            i += 1;
        }
        if kind(i) == Some(END) || kind(i).is_none() {
            return Ok(db.model);
        }
    }
}

pub(crate) fn parse(input: &ParseInput) -> Result<IshikawaModel, Error> {
    parse_ishikawa(input.source, input.title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nests_by_indentation() {
        let m = parse_ishikawa("ishikawa\n    Effect\n  A\n    a1\n      x\n  B\n b0\n", None).unwrap();
        let texts: Vec<&str> = m.nodes.iter().map(|n| n.text.as_str()).collect();
        assert_eq!(texts, ["Effect", "A", "a1", "x", "B", "b0"]);
        assert_eq!(m.nodes[0].children, [1, 4, 5]);
        assert_eq!(m.nodes[1].children, [2]);
        assert_eq!(m.nodes[2].children, [3]);
    }

    #[test]
    fn errors() {
        let e = parse_ishikawa("ishikawa\n", None).unwrap_err();
        assert!(e.message.ends_with("Expecting 'SPACELINE', 'SPACELIST', 'TEXT', got 'EOF'"), "{}", e.message);
        let e = parse_ishikawa("fish\n", None).unwrap_err();
        assert!(e.message.ends_with("Expecting 'SPACELINE', 'ISHIKAWA', got 'TEXT'"), "{}", e.message);
    }
}
