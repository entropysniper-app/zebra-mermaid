//! The C4 parser. Ported from Pele `src/diagrams/c4/parser.ts`.

use super::db::C4Db;
use super::lexer::*;
use super::{C4Attr, C4Named};
use crate::detect::DiagramType;
use crate::error::{syntax_error, Error};
use crate::util::chars::char_at;
use crate::util::js;

/// Mermaid's typeC4Shape for each shape keyword, indexed from `PERSON`.
const KINDS: [&str; 20] = [
    "person",
    "external_person",
    "system",
    "system_db",
    "system_queue",
    "external_system",
    "external_system_db",
    "external_system_queue",
    "container",
    "container_db",
    "container_queue",
    "external_container",
    "external_container_db",
    "external_container_queue",
    "component",
    "component_db",
    "component_queue",
    "external_component",
    "external_component_db",
    "external_component_queue",
];
const REL_TYPES: [&str; 8] = ["rel", "birel", "rel_u", "rel_d", "rel_l", "rel_r", "rel_b", "rel"];
const NODE_TYPES: [&str; 3] = ["node", "nodeL", "nodeR"];

const HEADERS: &str =
    "'direction_tb', 'direction_bt', 'direction_rl', 'direction_lr', 'C4_CONTEXT', 'C4_CONTAINER', 'C4_COMPONENT', 'C4_DYNAMIC', 'C4_DEPLOYMENT'";
const ATTRIBUTES: &str = "'STR', 'STR_KEY', 'ATTRIBUTE', 'ATTRIBUTE_EMPTY'";

#[derive(Clone, Copy, PartialEq, Eq)]
enum P {
    /// A statement is required.
    Item,
    /// Only a diagram statement may come next: the first one in a boundary, or one written
    /// straight after a title.
    Diagram,
    ItemOrEnd,
    AfterOther,
    AfterDiagram,
}

fn is_other(t: u8) -> bool {
    t == TITLE || t == ACC_DESCRIPTION || t == ACC_TITLE || t == ACC_DESCR || t == ACC_DESCR_MULTILINE_VALUE
}

fn is_diagram(t: u8) -> bool {
    (PERSON..=UPDATE_LAYOUT_CONFIG).contains(&t)
}

/// `s.substring(k)` where `s` starts with an ASCII word of `k - 1` characters and one white
/// space character, which is a single UTF-16 unit.
fn after_word(s: &str, word_len: usize) -> &str {
    let space = char_at(s, word_len).map_or(1, char::len_utf8);
    s.get(word_len + space..).unwrap_or("")
}

struct Parser<'a> {
    src: &'a str,
    tokens: Tokens,
    i: usize,
}

impl<'a> Parser<'a> {
    fn kind(&self) -> u8 {
        // The token list ends with `$end` or an error, and the parser stops at either.
        self.tokens.types.get(self.i).copied().unwrap_or(END)
    }

    fn text(&self, at: usize) -> &'a str {
        &self.src[self.tokens.starts[at]..self.tokens.ends[at]]
    }

    fn fail(&self, expected: &str) -> Error {
        let t = self.kind();
        let at = self.tokens.starts.get(self.i).copied().unwrap_or(self.src.len());
        if t == ERROR {
            return syntax_error(DiagramType::C4, self.src, at, "", true);
        }
        syntax_error(DiagramType::C4, self.src, at, &format!("Expecting {}, got '{}'", expected, TOKEN_NAMES[t as usize]), false)
    }

    fn expect(&mut self, kind: u8) -> Result<(), Error> {
        if self.kind() != kind {
            return Err(self.fail(&format!("'{}'", TOKEN_NAMES[kind as usize])));
        }
        self.i += 1;
        Ok(())
    }

    fn attributes(&mut self) -> Result<Vec<C4Attr>, Error> {
        let mut attrs = Vec::new();
        loop {
            let t = self.kind();
            if t == STR {
                attrs.push(C4Attr::Str(js::trim(self.text(self.i)).to_string()));
                self.i += 1;
            } else if t == ATTRIBUTE_EMPTY {
                attrs.push(C4Attr::Str(String::new()));
                self.i += 1;
            } else if t == STR_KEY {
                self.i += 1;
                if self.kind() != STR_VALUE {
                    return Err(self.fail("'STR_VALUE'"));
                }
                attrs.push(C4Attr::Named(C4Named {
                    key: js::trim(self.text(self.i - 1)).to_string(),
                    value: js::trim(self.text(self.i)).to_string(),
                }));
                self.i += 1;
            } else {
                break;
            }
        }
        if attrs.is_empty() {
            return Err(self.fail(ATTRIBUTES));
        }
        Ok(attrs)
    }

    fn other(&mut self, t: u8, db: &mut C4Db) -> Result<(), Error> {
        if t == TITLE {
            db.set_title(after_word(self.text(self.i), 5));
            self.i += 1;
        } else if t == ACC_DESCRIPTION {
            db.set_acc_description(after_word(self.text(self.i), 14));
            self.i += 1;
        } else if t == ACC_DESCR_MULTILINE_VALUE {
            db.set_acc_description(js::trim(self.text(self.i)));
            self.i += 1;
        } else {
            self.i += 1;
            if t == ACC_TITLE {
                if self.kind() != ACC_TITLE_VALUE {
                    return Err(self.fail("'acc_title_value'"));
                }
                // The grammar hands `accTitle:` to setTitle, so it becomes the visible title.
                db.set_title(js::trim(self.text(self.i)));
                self.i += 1;
            } else {
                if self.kind() != ACC_DESCR_VALUE {
                    return Err(self.fail("'acc_descr_value'"));
                }
                db.set_acc_description(js::trim(self.text(self.i)));
                self.i += 1;
            }
        }
        Ok(())
    }

    /// Returns true when the statement opens a boundary.
    fn diagram(&mut self, t: u8, db: &mut C4Db) -> Result<bool, Error> {
        self.i += 1;
        let mut attrs = self.attributes()?;
        if t <= SYSTEM_EXT_QUEUE {
            db.add_person_or_system(KINDS[(t - PERSON) as usize], &attrs);
        } else if t <= CONTAINER_EXT_QUEUE {
            db.add_container(KINDS[(t - PERSON) as usize], &attrs);
        } else if t <= COMPONENT_EXT_QUEUE {
            db.add_component(KINDS[(t - PERSON) as usize], &attrs);
        } else if t <= NODE_R {
            // `attrs.splice(2, 0, type)`: inserted at the end when there are fewer than two.
            let fixed = match t {
                ENTERPRISE_BOUNDARY => Some("ENTERPRISE"),
                SYSTEM_BOUNDARY => Some("SYSTEM"),
                CONTAINER_BOUNDARY => Some("CONTAINER"),
                _ => None,
            };
            if let Some(kind) = fixed {
                attrs.insert(attrs.len().min(2), C4Attr::Str(kind.to_string()));
            }
            if t >= NODE {
                db.add_deployment_node(NODE_TYPES[(t - NODE) as usize], &attrs);
            } else if t == CONTAINER_BOUNDARY {
                db.add_container_boundary(&attrs);
            } else {
                db.add_person_or_system_boundary(&attrs, "system");
            }
            if self.kind() == LBRACE {
                self.i += 1;
                self.expect(NEWLINE)?;
            } else if self.kind() == NEWLINE {
                self.i += 1;
                self.expect(LBRACE)?;
                if self.kind() == NEWLINE {
                    self.i += 1;
                }
            } else {
                return Err(self.fail("'NEWLINE', 'LBRACE'"));
            }
            return Ok(true);
        } else if t <= REL_INDEX {
            // RelIndex takes a sequence number first, which Mermaid ignores.
            if t == REL_INDEX {
                attrs.remove(0);
            }
            db.add_rel(REL_TYPES[(t - REL) as usize], &attrs);
        } else if t == UPDATE_EL_STYLE {
            db.update_el_style(&attrs);
        } else if t == UPDATE_REL_STYLE {
            db.update_rel_style(&attrs);
        } else {
            db.update_layout_config(&attrs);
        }
        Ok(false)
    }
}

/// Accepts what the parser Jison generates from c4Diagram.jison accepts, and calls the model
/// builder in the order its semantic actions do. Boundaries nest through a counter, not
/// recursion.
pub(super) fn parse_c4(src: &str, db: &mut C4Db) -> Result<(), Error> {
    let mut p = Parser { src, tokens: tokenize(src), i: 0 };

    let header = p.kind();
    if !(C4_CONTEXT..=C4_DEPLOYMENT).contains(&header) {
        return Err(p.fail(HEADERS));
    }
    p.i += 1;
    p.expect(NEWLINE)?;

    let mut depth = 0usize;
    let mut state = P::Item;
    loop {
        let t = p.kind();
        if state == P::AfterOther || state == P::AfterDiagram {
            if t == NEWLINE {
                p.i += 1;
                state = P::ItemOrEnd;
                continue;
            }
            if state == P::AfterOther && is_diagram(t) {
                state = P::Diagram;
                continue;
            }
        } else if is_diagram(t) {
            if p.diagram(t, db)? {
                depth += 1;
                state = P::Diagram;
            } else {
                state = P::AfterDiagram;
            }
            continue;
        } else if is_other(t) && state != P::Diagram {
            p.other(t, db)?;
            state = P::AfterOther;
            continue;
        } else if state != P::ItemOrEnd {
            return Err(p.fail("'title', 'accDescription', 'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'PERSON', 'REL'"));
        }

        if depth > 0 {
            if t != RBRACE {
                return Err(p.fail("'NEWLINE', 'RBRACE'"));
            }
            p.i += 1;
            db.pop_boundary_parse_stack();
            depth -= 1;
            state = P::AfterDiagram;
        } else {
            if t != EOF {
                return Err(p.fail("'NEWLINE', 'EOF'"));
            }
            p.i += 1;
            p.expect(END)?;
            db.set_c4_type(p.text(0));
            return Ok(());
        }
    }
}
