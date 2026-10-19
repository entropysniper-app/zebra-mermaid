//! What the quadrant chart parser builds. Ported from Pele `src/diagrams/quadrant/db.ts`.

use super::{PointStyle, QuadrantModel, QuadrantPoint, QuadrantText};
use crate::detect::DiagramType;
use crate::error::Error;
use crate::util::js;

fn invalid(style: &str, value: &str, kind: &str) -> Error {
    Error::semantic(format!("value for {} {} is invalid, please use a valid {}", style, value, kind)).with_type(DiagramType::QuadrantChart)
}

/// `/^\d+$/`
fn is_number(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit())
}

/// `/^#?([\dA-Fa-f]{6}|[\dA-Fa-f]{3})$/`
fn is_hex(s: &str) -> bool {
    let body = s.strip_prefix('#').unwrap_or(s);
    (body.len() == 6 || body.len() == 3) && body.bytes().all(|c| c.is_ascii_hexdigit())
}

/// `/^\d+px$/`
fn is_pixels(s: &str) -> bool {
    s.strip_suffix("px").is_some_and(is_number)
}

/// The first two parts of Mermaid's `style.trim().split(/\s*:\s*/)`, found in one pass.
fn key_value(style: &str) -> (&str, &str) {
    let s = js::trim(style);
    let Some(colon) = s.find(':') else { return (s, "undefined") };
    let rest = js::trim_start(&s[colon + 1..]);
    let value = match rest.find(':') {
        None => rest,
        Some(next) => js::trim_end(&rest[..next]),
    };
    (js::trim_end(&s[..colon]), value)
}

pub fn parse_styles(styles: &[String]) -> Result<PointStyle, Error> {
    let mut out = PointStyle::default();
    for style in styles {
        let (key, value) = key_value(style);
        match key {
            "radius" => {
                if !is_number(value) {
                    return Err(invalid(key, value, "number"));
                }
                out.radius = Some(js::parse_int(value, 0));
            }
            "color" => {
                if !is_hex(value) {
                    return Err(invalid(key, value, "hex code"));
                }
                out.color = Some(value.to_string());
            }
            "stroke-color" => {
                if !is_hex(value) {
                    return Err(invalid(key, value, "hex code"));
                }
                out.stroke_color = Some(value.to_string());
            }
            "stroke-width" => {
                if !is_pixels(value) {
                    return Err(invalid(key, value, "number of pixels (eg. 10px)"));
                }
                out.stroke_width = Some(value.to_string());
            }
            _ => {
                return Err(Error::semantic(format!("style named {} is not supported.", key)).with_type(DiagramType::QuadrantChart));
            }
        }
    }
    Ok(out)
}

fn trimmed(label: QuadrantText) -> QuadrantText {
    QuadrantText { text: js::trim(&label.text).to_string(), markdown: label.markdown }
}

pub(crate) struct QuadrantDb {
    pub model: QuadrantModel,
}

impl QuadrantDb {
    pub fn new(title: Option<&str>) -> Self {
        QuadrantDb {
            model: QuadrantModel {
                title: title.map(str::to_string),
                acc_title: None,
                acc_descr: None,
                quadrants: [None, None, None, None],
                x_axis_left: None,
                x_axis_right: None,
                y_axis_bottom: None,
                y_axis_top: None,
                points: Vec::new(),
                classes: indexmap::IndexMap::new(),
            },
        }
    }

    /// Quadrant `k` (0 to 3) is Mermaid's quadrant `k + 1`.
    pub fn set_quadrant_text(&mut self, k: usize, label: QuadrantText) {
        self.model.quadrants[k] = Some(trimmed(label));
    }

    pub fn set_x_axis_left_text(&mut self, label: QuadrantText) {
        self.model.x_axis_left = Some(trimmed(label));
    }

    pub fn set_x_axis_right_text(&mut self, label: QuadrantText) {
        self.model.x_axis_right = Some(trimmed(label));
    }

    pub fn set_y_axis_bottom_text(&mut self, label: QuadrantText) {
        self.model.y_axis_bottom = Some(trimmed(label));
    }

    pub fn set_y_axis_top_text(&mut self, label: QuadrantText) {
        self.model.y_axis_top = Some(trimmed(label));
    }

    pub fn add_point(&mut self, label: QuadrantText, class_name: String, x: &str, y: &str, styles: &[String]) -> Result<(), Error> {
        let style = parse_styles(styles)?;
        self.model.points.push(QuadrantPoint { x: js::number(x), y: js::number(y), text: trimmed(label), class_name, style });
        Ok(())
    }

    pub fn add_class(&mut self, class_name: String, styles: &[String]) -> Result<(), Error> {
        let style = parse_styles(styles)?;
        self.model.classes.insert(class_name, style);
        Ok(())
    }

    pub fn set_diagram_title(&mut self, text: &str) {
        self.model.title = Some(text.to_string());
    }

    pub fn set_acc_title(&mut self, text: &str) {
        self.model.acc_title = Some(js::trim_start(text).to_string());
    }

    pub fn set_acc_description(&mut self, text: &str) {
        self.model.acc_descr = Some(crate::diagrams::squeeze_line_starts(text));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles() {
        assert_eq!(key_value(" radius : 10 : x "), ("radius", "10"));
        assert_eq!(key_value("radius"), ("radius", "undefined"));
        let s = parse_styles(&["radius: 12".into(), "color: #ff0000".into(), "stroke-width: 3px".into()]).unwrap();
        assert_eq!(s.radius, Some(12.0));
        assert_eq!(s.color.as_deref(), Some("#ff0000"));
        assert_eq!(s.stroke_width.as_deref(), Some("3px"));
        let e = parse_styles(&["color: red".into()]).unwrap_err();
        assert_eq!(e.message, "value for color red is invalid, please use a valid hex code");
        let e = parse_styles(&["fill: #fff".into()]).unwrap_err();
        assert_eq!(e.message, "style named fill is not supported.");
        assert!(is_hex("abc") && is_hex("#A1b2C3") && !is_hex("#abcd") && !is_hex("#"));
        assert!(is_pixels("10px") && !is_pixels("px") && !is_pixels("1.5px"));
    }
}
