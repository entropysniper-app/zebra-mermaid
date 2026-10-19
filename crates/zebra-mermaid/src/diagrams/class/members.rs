//! Class member signatures and classifiers, ported from Pele members.ts.
use crate::diagrams::common::generics::parse_generic_types;
use crate::util::{chars::is_space, js};
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Member {
    pub id: String,
    pub method: bool,
    pub visibility: String,
    pub classifier: String,
    pub parameters: String,
    pub return_type: String,
}
fn visibility(c: char) -> bool {
    matches!(c, '#' | '+' | '~' | '-')
}
impl Member {
    pub fn new(input: &str, method: bool) -> Self {
        let mut m = Self { method, ..Default::default() };
        let mut found = false;
        if method {
            let mut start = 0;
            while start < input.len() {
                let end = input[start..].find(['\n', '\r', '\u{2028}', '\u{2029}']).map_or(input.len(), |i| start + i);
                if let Some(close) = input[start..end].rfind(')').map(|i| start + i) {
                    if let Some(open) = input[start..close].rfind('(').map(|i| start + i) {
                        if open > start {
                            let first = input[start..].chars().next().unwrap();
                            let visible = visibility(first) && open > start + 1;
                            if visible {
                                m.visibility = first.to_string();
                            }
                            m.id = input[if visible { start + 1 } else { start }..open].into();
                            m.parameters = js::trim(&input[open + 1..close]).into();
                            let mut rest = close + 1;
                            if let Some(next) = input[rest..].chars().next() {
                                if matches!(next, '$' | '*') || is_space(next) {
                                    m.classifier = js::trim(&next.to_string()).into();
                                    rest += next.len_utf8();
                                }
                            }
                            let stop = input[rest..].find(['\n', '\r', '\u{2028}', '\u{2029}']).map_or(input.len(), |i| rest + i);
                            m.return_type = js::trim(&input[rest..stop]).into();
                            if m.classifier.is_empty() {
                                if let Some(last) = m.return_type.chars().last().filter(|&c| matches!(c, '$' | '*')) {
                                    m.classifier = last.to_string();
                                    m.return_type.pop();
                                }
                            }
                            found = true;
                            break;
                        }
                    }
                }
                if end == input.len() {
                    break;
                }
                start = end + input[end..].chars().next().unwrap().len_utf8();
            }
        }
        if !found {
            m.method = false;
            let first = input.chars().next();
            let last = input.chars().last();
            if first.is_some_and(visibility) {
                m.visibility = first.unwrap().to_string();
            }
            if last.is_some_and(|c| matches!(c, '$' | '*')) {
                m.classifier = last.unwrap().to_string();
            }
            let a = usize::from(!m.visibility.is_empty());
            let b = input.len().saturating_sub(usize::from(!m.classifier.is_empty()));
            m.id = input[a.min(b)..b].into();
        }
        m.id = if m.id.starts_with(' ') { format!(" {}", js::trim(&m.id)) } else { js::trim(&m.id).into() };
        m
    }
    pub fn display(&self) -> String {
        let mut text = self.visibility.clone() + &parse_generic_types(&self.id);
        if self.method {
            text.push('(');
            text.push_str(&parse_generic_types(js::trim(&self.parameters)));
            text.push(')');
            if !self.return_type.is_empty() {
                text.push_str(" : ");
                text.push_str(&parse_generic_types(&self.return_type));
            }
        }
        js::trim(&text).into()
    }
}
