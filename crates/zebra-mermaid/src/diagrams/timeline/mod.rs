//! Timelines. Ported from Pele `src/diagrams/timeline/`.

mod lexer;
mod render;

use crate::detect::DiagramType;
use crate::diagrams::ParseInput;
use crate::error::{syntax_error, Error};
use crate::util::js;

pub(crate) use render::render;

use lexer::{
    tokenize_borrowed as tokenize, ACC_DESCR, ACC_DESCR_MULTILINE_VALUE, ACC_TITLE, END, EOF, EVENT, NEWLINE, PERIOD, SECTION, TIMELINE,
    TIMELINE_LR, TIMELINE_TD, TITLE, TOKEN_NAMES,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TimelinePeriod {
    pub text: String,
    pub events: Vec<String>,
    /// Name of the section in force when the period was declared, empty before the first one.
    pub section: String,
    /// Index into `sections` of that section, -1 before the first one.
    pub section_index: i64,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TimelineModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
    /// `LR` or `TD`.
    pub direction: String,
    pub sections: Vec<String>,
    pub periods: Vec<TimelinePeriod>,
}

/// Pele's `TimelineDb`: what the statements build.
struct TimelineDb {
    model: TimelineModel,
    current_section: String,
}

impl TimelineDb {
    fn add_section(&mut self, text: &str) {
        self.current_section = text.to_string();
        self.model.sections.push(text.to_string());
    }

    fn add_task(&mut self, period: &str) {
        self.model.periods.push(TimelinePeriod {
            text: period.to_string(),
            events: Vec::new(),
            section: self.current_section.clone(),
            section_index: self.model.sections.len() as i64 - 1,
        });
    }

    fn add_event(&mut self, event: &str) -> Result<(), Error> {
        // Mermaid fails here too, with a TypeError.
        let Some(period) = self.model.periods.last_mut() else {
            return Err(Error::semantic("An event needs a time period before it.").with_type(DiagramType::Timeline));
        };
        period.events.push(event.to_string());
        Ok(())
    }
}

/// `text.substring(k + 1)` where the token starts with a `k`-unit ASCII keyword and one white
/// space character: what follows that character.
fn after_space(text: &str, keyword: usize) -> &str {
    let rest = &text[keyword..];
    &rest[rest.chars().next().map_or(0, char::len_utf8)..]
}

const STATEMENT: &str =
    "'EOF', 'SPACE', 'NEWLINE', 'title', 'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'section', 'period', 'event'";

/// Pele's `parseTimeline`, which follows the parser Jison builds from Mermaid's grammar.
pub fn parse_timeline(src: &str, title: Option<&str>) -> Result<TimelineModel, Error> {
    let tokens = tokenize(src);
    let (types, texts, starts) = (&tokens.types, &tokens.texts, &tokens.starts);
    let kind = |i: usize| types.get(i).copied().unwrap_or(END);
    let fail = |i: usize, expected: &str| -> Error {
        let i = i.min(types.len() - 1);
        let got = if types[i] == END { "1" } else { TOKEN_NAMES[types[i]] };
        syntax_error(DiagramType::Timeline, src, starts[i], &format!("Expecting {expected}, got '{got}'"), false)
    };
    let text = |i: usize| texts.get(i).copied().unwrap_or("");

    let mut db = TimelineDb {
        model: TimelineModel {
            title: title.map(str::to_string),
            acc_title: None,
            acc_descr: None,
            direction: "LR".to_string(),
            sections: Vec::new(),
            periods: Vec::new(),
        },
        current_section: String::new(),
    };

    let head = kind(0);
    if head != TIMELINE && head != TIMELINE_LR && head != TIMELINE_TD {
        return Err(fail(0, "'timeline', 'timeline_lr', 'timeline_td'"));
    }
    if head != TIMELINE {
        db.model.direction = if head == TIMELINE_LR { "LR" } else { "TD" }.to_string();
    }

    let mut i = 1;
    loop {
        match kind(i) {
            NEWLINE => {}
            TITLE => db.model.title = Some(after_space(text(i), 5).to_string()),
            ACC_TITLE => {
                i += 1;
                // `setAccTitle(text.trim())` then strips leading white space, which is gone.
                db.model.acc_title = Some(js::trim(text(i)).to_string());
            }
            ACC_DESCR => {
                i += 1;
                db.model.acc_descr = Some(crate::diagrams::squeeze_line_starts(js::trim(text(i))));
            }
            ACC_DESCR_MULTILINE_VALUE => db.model.acc_descr = Some(crate::diagrams::squeeze_line_starts(js::trim(text(i)))),
            SECTION => db.add_section(after_space(text(i), 7)),
            PERIOD => db.add_task(text(i)),
            EVENT => db.add_event(after_space(text(i), 1))?,
            EOF => return Ok(db.model),
            _ => return Err(fail(i, STATEMENT)),
        }
        i += 1;
    }
}

pub(crate) fn parse(input: &ParseInput) -> Result<TimelineModel, Error> {
    parse_timeline(input.source, input.title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_periods_and_sections() {
        let m = parse_timeline("timeline TD\ntitle History\n1990 : a\nsection S\n2000 : b : c\n", None).unwrap();
        assert_eq!(m.direction, "TD");
        assert_eq!(m.title.as_deref(), Some("History"));
        assert_eq!(m.sections, ["S"]);
        assert_eq!(m.periods[0].text, "1990 ");
        assert_eq!(m.periods[0].section_index, -1);
        assert_eq!(m.periods[1].events, ["b ", "c"]);
        assert_eq!(m.periods[1].section, "S");
    }

    #[test]
    fn errors() {
        let e = parse_timeline("timeline\n: a\n", None).unwrap_err();
        assert_eq!(e.message, "An event needs a time period before it.");
        let e = parse_timeline("timeline\naccTitle:  \n", None).unwrap_err();
        assert!(e.message.ends_with("got '1'"), "{}", e.message);
        let e = parse_timeline("timelinex\n", None).unwrap_err();
        assert!(e.message.ends_with("Expecting 'timeline', 'timeline_lr', 'timeline_td', got 'period'"), "{}", e.message);
    }
}
