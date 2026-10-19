//! The journey parser, ported from Pele `src/diagrams/journey/parser.ts`.

use super::lexer::{self, tokenize};
use super::JourneyDb;
use crate::detect::DiagramType;
use crate::error::{syntax_error, Error};
use crate::util::js;

const STATEMENT: &str = "'EOF', 'SPACE', 'NEWLINE', 'title', 'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'section', 'taskName'";

/// The text after the keyword and the white space character that follows it. Pele takes
/// `substring(n)` of the token, the keyword being `n - 1` ASCII units and the white space
/// one unit.
fn after_keyword(text: &str, keyword: usize) -> &str {
    let space = text[keyword..].chars().next().map_or(0, char::len_utf8);
    &text[keyword + space..]
}

pub fn parse_journey(src: &str, db: &mut JourneyDb) -> Result<(), Error> {
    let t = tokenize(src);
    let (types, texts, starts) = (&t.types, &t.texts, &t.starts);

    let fail = |i: usize, expected: &str| -> Error {
        let got = if types[i] == lexer::END { "1" } else { lexer::TOKEN_NAMES[types[i]] };
        syntax_error(DiagramType::Journey, src, starts[i], &format!("Expecting {}, got '{}'", expected, got), false)
    };

    if types[0] != lexer::JOURNEY {
        return Err(fail(0, "'journey'"));
    }

    let mut i = 1;
    loop {
        let text = &texts[i];
        match types[i] {
            lexer::NEWLINE => {}
            lexer::TITLE => db.set_diagram_title(after_keyword(text, 5)),
            lexer::ACC_TITLE => {
                i += 1;
                db.set_acc_title(js::trim(&texts[i]));
            }
            lexer::ACC_DESCR => {
                i += 1;
                db.set_acc_description(js::trim(&texts[i]));
            }
            lexer::ACC_DESCR_MULTILINE_VALUE => db.set_acc_description(js::trim(text)),
            lexer::SECTION => db.add_section(after_keyword(text, 7)),
            lexer::TASK_NAME => {
                i += 1;
                if types[i] != lexer::TASK_DATA {
                    return Err(fail(i, "'taskData'"));
                }
                db.add_task(text, &texts[i]);
            }
            lexer::EOF => return Ok(()),
            _ => return Err(fail(i, STATEMENT)),
        }
        i += 1;
    }
}
