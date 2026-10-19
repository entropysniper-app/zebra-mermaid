//! Gantt parser. Ported from Pele `src/diagrams/gantt/parser.ts`.

use super::db::Builder;
use super::lexer::{t, tokenize, Token};
use crate::detect::DiagramType;
use crate::error::{syntax_error, Error};
use crate::util::js;

const WEEKDAYS: [&str; 7] = ["monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday"];

const LINE: &str = "'EOF', 'SPACE', 'NL', 'weekday_monday', 'weekday_tuesday', 'weekday_wednesday', 'weekday_thursday', \
'weekday_friday', 'weekday_saturday', 'weekday_sunday', 'weekend_friday', 'weekend_saturday', 'dateFormat', \
'inclusiveEndDates', 'topAxis', 'axisFormat', 'tickInterval', 'excludes', 'includes', 'todayMarker', 'title', \
'acc_title', 'acc_descr', 'acc_descr_multiline_value', 'section', 'taskTxt', 'click'";

/// Tokens that can start a line, which are also the only ones that can follow a complete
/// statement.
fn starts_line(kind: u8) -> bool {
    (t::WEEKDAY_MONDAY..=t::WEEKDAY_SUNDAY).contains(&kind)
        || matches!(kind, t::WEEKEND_FRIDAY | t::WEEKEND_SATURDAY)
        || matches!(
            kind,
            t::EOF
                | t::NL
                | t::DATE_FORMAT
                | t::INCLUSIVE_END_DATES
                | t::TOP_AXIS
                | t::AXIS_FORMAT
                | t::TICK_INTERVAL
                | t::EXCLUDES
                | t::INCLUDES
                | t::TODAY_MARKER
                | t::TITLE
                | t::ACC_TITLE
                | t::ACC_DESCR
                | t::ACC_DESCR_MULTILINE_VALUE
                | t::SECTION
                | t::TASK_TXT
                | t::CLICK
        )
}

/// `text.slice(n)` with `n` in UTF-16 code units.
fn after(text: &str, n: usize) -> &str {
    &text[js::byte_at16(text, n)..]
}

pub(crate) fn parse_gantt(src: &str, db: &mut Builder) -> Result<(), Error> {
    let tokens: Vec<Token> = tokenize(src);
    let mut i = 0;
    let kind = |i: usize| tokens.get(i).map_or(t::END, |tk| tk.kind);
    let text = |i: usize| tokens.get(i).map_or("", |tk| &src[tk.start..tk.end]);
    let fail = |i: usize, expected: &str| -> Error {
        let k = kind(i);
        let got = if k == t::END { "1" } else { t::NAMES[k as usize] };
        let at = tokens.get(i).map_or(src.len(), |tk| tk.start);
        syntax_error(DiagramType::Gantt, src, at, &format!("Expecting {}, got '{}'", expected, got), false)
    };
    // Mermaid's parser runs a statement's action only after seeing that the next token can
    // follow it.
    let end = |i: usize, also: &str| -> Result<(), Error> {
        if starts_line(kind(i)) {
            Ok(())
        } else {
            Err(fail(i, &format!("{}{}", LINE, also)))
        }
    };

    if kind(i) != t::GANTT {
        return Err(fail(i, "'gantt'"));
    }
    i += 1;

    loop {
        let k = kind(i);
        let tx = text(i);
        if !starts_line(k) {
            return Err(fail(i, LINE));
        }
        i += 1;
        match k {
            t::EOF => {
                if kind(i) == t::END {
                    return Ok(());
                }
                end(i, "")?;
            }
            t::NL => {}
            t::DATE_FORMAT => {
                end(i, "")?;
                db.date_format = after(tx, 11).to_string();
            }
            t::INCLUSIVE_END_DATES => {
                end(i, "")?;
                db.inclusive_end_dates = true;
            }
            t::TOP_AXIS => {
                end(i, "")?;
                db.top_axis = true;
            }
            t::AXIS_FORMAT => {
                end(i, "")?;
                db.axis_format = after(tx, 11).to_string();
            }
            t::TICK_INTERVAL => {
                end(i, "")?;
                db.tick_interval = Some(after(tx, 13).to_string());
            }
            t::EXCLUDES => {
                end(i, "")?;
                db.set_excludes(after(tx, 9));
            }
            t::INCLUDES => {
                end(i, "")?;
                db.set_includes(after(tx, 9));
            }
            t::TODAY_MARKER => {
                end(i, "")?;
                db.today_marker = after(tx, 12).to_string();
            }
            t::WEEKEND_FRIDAY => {
                end(i, "")?;
                db.weekend = "friday".to_string();
            }
            t::WEEKEND_SATURDAY => {
                end(i, "")?;
                db.weekend = "saturday".to_string();
            }
            t::TITLE => {
                end(i, "")?;
                db.title = Some(after(tx, 6).to_string());
            }
            t::ACC_TITLE => {
                if kind(i) != t::ACC_TITLE_VALUE {
                    return Err(fail(i, "'acc_title_value'"));
                }
                let value = text(i);
                i += 1;
                end(i, "")?;
                db.set_acc_title(js::trim(value));
            }
            t::ACC_DESCR => {
                if kind(i) != t::ACC_DESCR_VALUE {
                    return Err(fail(i, "'acc_descr_value'"));
                }
                let value = text(i);
                i += 1;
                end(i, "")?;
                db.set_acc_description(js::trim(value));
            }
            t::ACC_DESCR_MULTILINE_VALUE => {
                end(i, "")?;
                db.set_acc_description(js::trim(tx));
            }
            t::SECTION => {
                end(i, "")?;
                db.add_section(after(tx, 8));
            }
            t::TASK_TXT => {
                if kind(i) != t::TASK_DATA {
                    return Err(fail(i, "'taskData'"));
                }
                let data = text(i);
                i += 1;
                end(i, "")?;
                db.add_task(tx, data)?;
            }
            t::CLICK => {
                let mut name: Option<&str> = None;
                let mut args: Option<&str> = None;
                let mut href: Option<&str> = None;
                if kind(i) == t::CALLBACKNAME {
                    name = Some(text(i));
                    i += 1;
                    if kind(i) == t::CALLBACKARGS {
                        args = Some(text(i));
                        i += 1;
                    }
                    if kind(i) == t::HREF {
                        href = Some(text(i));
                        i += 1;
                        end(i, "")?;
                    } else {
                        end(i, if args.is_none() { ", 'callbackargs', 'href'" } else { ", 'href'" })?;
                    }
                } else if kind(i) == t::HREF {
                    href = Some(text(i));
                    i += 1;
                    if kind(i) == t::CALLBACKNAME {
                        name = Some(text(i));
                        i += 1;
                        if kind(i) == t::CALLBACKARGS {
                            args = Some(text(i));
                            i += 1;
                            end(i, "")?;
                        } else {
                            end(i, ", 'callbackargs'")?;
                        }
                    } else {
                        end(i, ", 'callbackname'")?;
                    }
                } else {
                    return Err(fail(i, "'callbackname', 'href'"));
                }
                let _ = args;
                if name.is_some() {
                    db.set_click_event(tx);
                }
                if let Some(href) = href {
                    db.set_link(tx, href);
                }
            }
            _ => {
                end(i, "")?;
                db.weekday = Some(WEEKDAYS[(k - t::WEEKDAY_MONDAY) as usize].to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Result<Builder, Error> {
        let mut db = Builder::default();
        parse_gantt(src, &mut db)?;
        Ok(db)
    }

    #[test]
    fn statements() {
        let db = parse("gantt\n  title  Plan\n  dateFormat YYYY-MM-DD\n  section S\n  A :a1, 2014-01-01, 3d\n").unwrap();
        assert_eq!(db.title.as_deref(), Some(" Plan"));
        assert_eq!(db.date_format, "YYYY-MM-DD");
        assert_eq!(db.sections, vec!["S".to_string()]);
        assert_eq!(db.tasks.len(), 1);
        assert_eq!(db.tasks[0].id, "a1");
    }

    #[test]
    fn errors() {
        let e = parse("gantt\n  A task\n").unwrap_err();
        assert_eq!(e.message, "Parse error on line 2:\n  A task\n--------^\nExpecting 'taskData', got 'NL'");
        let e = parse("chart\n").unwrap_err();
        assert!(e.message.ends_with("Expecting 'gantt', got 'taskTxt'"), "{}", e.message);
    }
}
