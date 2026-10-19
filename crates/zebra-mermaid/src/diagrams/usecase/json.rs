//! Ordered JSON nodes use an arena and an explicit stack, including on deeply nested input.
use super::{
    lexer::{line_column, Source},
    types::{JsonValue, OrderedJson},
};
use crate::{ConfigValue, DiagramType, Error, ErrorCode};
use indexmap::IndexMap;
#[derive(Clone, Copy)]
enum Mode {
    ObjectKey(bool),
    ArrayValue(bool),
    After,
}
struct Frame {
    id: usize,
    pointer: String,
    key: String,
    from: usize,
    ranges: IndexMap<String, (usize, usize)>,
    mode: Mode,
}
struct Reader {
    source: Source,
    at: usize,
    origin: (usize, usize),
    out: OrderedJson,
    created: Vec<String>,
}
impl Reader {
    fn code(&self) -> u16 {
        self.source.code(self.at)
    }
    fn skip(&mut self) {
        while matches!(self.code(), 32 | 9 | 10 | 13) {
            self.at += 1;
        }
    }
    fn error(&self, message: String, at: usize) -> Error {
        let (mut line, mut column) = self.origin;
        for i in 0..at.min(self.source.len()) {
            let c = self.source.code(i);
            if c == 13 {
                line += 1;
                column = 1;
            } else if c == 10 {
                if i == 0 || self.source.code(i - 1) != 13 {
                    line += 1;
                    column = 1;
                }
            } else {
                column += 1;
            }
        }
        let mut e = Error::new(format!("Invalid JSON: {message} (line {line}, column {column})"), ErrorCode::Syntax)
            .with_type(DiagramType::Usecase);
        e.line = line;
        e.column = column;
        e
    }
    fn positioned(&self, detail: &str, at: usize) -> Error {
        let (l, c) = line_column(&self.source, at);
        self.error(format!("{detail} in JSON at position {at} (line {l} column {c})"), at)
    }
    fn unexpected(&self, at: usize) -> Error {
        if at >= self.source.len() {
            return self.error("Unexpected end of JSON input".into(), at);
        }
        let token = self.source.slice(at, at + 1);
        let n = self.source.len();
        let shown = if n <= 20 {
            format!("\"{}\"", self.source.slice(0, n))
        } else if at < 10 {
            format!("\"{}\"...", self.source.slice(0, 10))
        } else {
            let end = (at + 10).min(n);
            format!("...\"{}\"{}", self.source.slice(at - 10, end), if end < n { "..." } else { "" })
        };
        self.error(format!("Unexpected token '{token}', {shown} is not valid JSON"), at)
    }
    fn string(&mut self) -> Result<String, Error> {
        self.at += 1;
        let mut value = vec![];
        while self.at < self.source.len() {
            let at = self.at;
            let c = self.code();
            self.at += 1;
            if c == 34 {
                return Ok(String::from_utf16_lossy(&value));
            }
            if c < 32 {
                return Err(self.positioned("Bad control character in string literal", at));
            }
            if c != 92 {
                value.push(c);
                continue;
            }
            let at = self.at;
            let escaped = self.code();
            self.at += 1;
            value.push(match escaped {
                34 | 92 | 47 => escaped,
                98 => 8,
                102 => 12,
                110 => 10,
                114 => 13,
                116 => 9,
                117 => {
                    let mut unit = 0;
                    for _ in 0..4 {
                        let c = self.code();
                        let digit = match c {
                            48..=57 => c - 48,
                            65..=70 => c - 65 + 10,
                            97..=102 => c - 97 + 10,
                            _ => return Err(self.positioned("Bad Unicode escape", self.at)),
                        };
                        unit = unit * 16 + digit;
                        self.at += 1;
                    }
                    unit
                }
                _ => return Err(self.positioned("Bad escaped character", at)),
            });
        }
        Err(self.positioned("Unterminated string", self.at))
    }
    fn number(&mut self) -> Result<f64, Error> {
        let start = self.at;
        if self.code() == 45 {
            self.at += 1;
            if !(48..=57).contains(&self.code()) {
                return Err(self.positioned("No number after minus sign", self.at));
            }
        }
        if self.code() == 48 {
            self.at += 1;
        } else {
            while (48..=57).contains(&self.code()) {
                self.at += 1;
            }
        }
        if self.code() == 46 {
            self.at += 1;
            if !(48..=57).contains(&self.code()) {
                return Err(self.positioned("Unterminated fractional number", self.at));
            }
            while (48..=57).contains(&self.code()) {
                self.at += 1;
            }
        }
        if self.code() | 32 == 101 {
            self.at += 1;
            if matches!(self.code(), 43 | 45) {
                self.at += 1;
            }
            if !(48..=57).contains(&self.code()) {
                return Err(self.positioned("Exponent part is missing a number", self.at));
            }
            while (48..=57).contains(&self.code()) {
                self.at += 1;
            }
        }
        Ok(self.source.slice(start, self.at).parse().unwrap_or(f64::NAN))
    }
    fn value(&mut self, pointer: String, stack: &mut Vec<Frame>) -> Result<usize, Error> {
        self.skip();
        let id = self.out.values.len();
        let mut mode = None;
        let value = match self.code() {
            123 => {
                self.at += 1;
                self.out.property_order.insert(pointer.clone(), vec![]);
                self.created.push(pointer.clone());
                mode = Some(Mode::ObjectKey(true));
                JsonValue::Object(IndexMap::new())
            }
            91 => {
                self.at += 1;
                mode = Some(Mode::ArrayValue(true));
                JsonValue::Array(vec![])
            }
            34 => JsonValue::Scalar(ConfigValue::String(self.string()?)),
            45 | 48..=57 => JsonValue::Scalar(ConfigValue::Number(self.number()?)),
            116 | 102 | 110 => {
                let word = if self.code() == 116 {
                    "true"
                } else if self.code() == 102 {
                    "false"
                } else {
                    "null"
                };
                for b in word.bytes() {
                    if self.code() != b as u16 {
                        return Err(self.unexpected(self.at));
                    }
                    self.at += 1;
                }
                JsonValue::Scalar(match word {
                    "true" => ConfigValue::Bool(true),
                    "false" => ConfigValue::Bool(false),
                    _ => ConfigValue::Null,
                })
            }
            _ => return Err(self.unexpected(self.at)),
        };
        self.out.values.push(value);
        if let Some(mode) = mode {
            stack.push(Frame { id, pointer, key: String::new(), from: 0, ranges: IndexMap::new(), mode });
        }
        Ok(id)
    }
    fn parse(mut self) -> Result<OrderedJson, Error> {
        let mut stack = vec![];
        let root = self.value(String::new(), &mut stack)?;
        self.out.root = root;
        while !stack.is_empty() {
            self.skip();
            let last = stack.len() - 1;
            let mode = stack[last].mode;
            let id = stack[last].id;
            match mode {
                Mode::ObjectKey(first) => {
                    if first && self.code() == 125 {
                        self.at += 1;
                        stack.pop();
                        continue;
                    }
                    if self.code() != 34 {
                        return Err(self.positioned(
                            if first { "Expected property name or '}'" } else { "Expected double-quoted property name" },
                            self.at,
                        ));
                    }
                    let key = self.string()?;
                    if let Some(&(from, to)) = stack[last].ranges.get(&key) {
                        for k in from..to {
                            self.out.property_order.shift_remove(&self.created[k]);
                        }
                    } else {
                        self.out.property_order.get_mut(&stack[last].pointer).unwrap().push(key.clone());
                    }
                    stack[last].key = key.clone();
                    stack[last].from = self.created.len();
                    self.skip();
                    if self.code() != 58 {
                        return Err(self.positioned("Expected ':' after property name", self.at));
                    }
                    self.at += 1;
                    stack[last].mode = Mode::After;
                    let pointer = format!("{}/{}", stack[last].pointer, key.replace('~', "~0").replace('/', "~1"));
                    let value = self.value(pointer, &mut stack)?;
                    let JsonValue::Object(object) = &mut self.out.values[id] else { unreachable!() };
                    object.insert(key, value);
                }
                Mode::ArrayValue(first) => {
                    if first && self.code() == 93 {
                        self.at += 1;
                        stack.pop();
                        continue;
                    }
                    let JsonValue::Array(items) = &self.out.values[id] else { unreachable!() };
                    let pointer = format!("{}/{}", stack[last].pointer, items.len());
                    stack[last].mode = Mode::After;
                    let value = self.value(pointer, &mut stack)?;
                    let JsonValue::Array(items) = &mut self.out.values[id] else { unreachable!() };
                    items.push(value);
                }
                Mode::After => {
                    let object = matches!(self.out.values[id], JsonValue::Object(_));
                    if object {
                        let key = stack[last].key.clone();
                        let from = stack[last].from;
                        stack[last].ranges.insert(key, (from, self.created.len()));
                    }
                    let close = if object { 125 } else { 93 };
                    if self.code() == close {
                        self.at += 1;
                        stack.pop();
                    } else if self.code() == 44 {
                        self.at += 1;
                        stack[last].mode = if object { Mode::ObjectKey(false) } else { Mode::ArrayValue(false) };
                    } else {
                        return Err(self.positioned(
                            if object { "Expected ',' or '}' after property value" } else { "Expected ',' or ']' after array element" },
                            self.at,
                        ));
                    }
                }
            }
        }
        self.skip();
        if self.at != self.source.len() {
            return Err(self.positioned("Unexpected non-whitespace character after JSON", self.at));
        }
        if !matches!(self.out.values[root], JsonValue::Object(_)) {
            let (line, column) = self.origin;
            let mut error = Error::new(format!("JSON value must have an object root (line {line}, column {column})"), ErrorCode::Syntax)
                .with_type(DiagramType::Usecase);
            error.line = line;
            error.column = column;
            return Err(error);
        }
        Ok(self.out)
    }
}
pub fn parse_ordered_json(text: &str, origin: (usize, usize)) -> Result<OrderedJson, Error> {
    Reader { source: Source::new(text), at: 0, origin, out: OrderedJson::default(), created: vec![] }.parse()
}
