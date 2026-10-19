//! Zebra's semantic palette: the data contract of `MermaidRenderPalette` (Swift) and
//! `MermaidPalette` (Zebra's native renderer), field for field.
//!
//! The JSON form uses camelCase names: 14 required colors and the optional `edge`,
//! `groupBackground`, `groupBorder` and `series` (exactly six colors). Colors are canonical
//! opaque `#RRGGBB` with upper-case digits. Two different defaulting rules exist and are kept
//! apart:
//!
//! - Zebra's Swift initializer fills the optional fields before encoding
//!   ([`MermaidPalette::swift_completed`]): edge = secondaryText, groupBackground = background,
//!   groupBorder = border, series = [accent, successText, warningText, dangerText, text,
//!   secondaryText]. Normal Zebra requests therefore carry every field.
//! - A caller of the native JSON boundary may leave the optional fields out; the renderer then
//!   derives them as Zebra's native renderer does (see [`super::ResolvedTheme`]).

use std::fmt;

use super::color::{mix_hex, Rgb};
use crate::config::json::{parse_json_limited, parse_json_members};
use crate::config::ConfigValue;

/// Light or dark: which default palette applies when the caller gives none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum MermaidThemeMode {
    #[default]
    Light,
    Dark,
}

impl MermaidThemeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            MermaidThemeMode::Light => "light",
            MermaidThemeMode::Dark => "dark",
        }
    }
}

/// Whether the SVG paints its own canvas. Independent of the palette: a transparent canvas
/// still uses `palette.background` for internal fills and for contrast decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum CanvasBackground {
    #[default]
    Opaque,
    Transparent,
}

impl CanvasBackground {
    pub fn as_str(self) -> &'static str {
        match self {
            CanvasBackground::Opaque => "opaque",
            CanvasBackground::Transparent => "transparent",
        }
    }
}

/// How colors are written into the SVG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum ColorMode {
    /// Pele's output: `--pele-*` custom properties with fallbacks, for a page to theme.
    #[default]
    Css,
    /// Concrete colors for every role; no `var()`, `color-mix()` or other CSS at run time.
    Resolved,
}

impl ColorMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ColorMode::Css => "css",
            ColorMode::Resolved => "resolved",
        }
    }
}

/// Zebra's semantic colors.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct MermaidPalette {
    pub background: String,
    pub surface: String,
    pub surface_secondary: String,
    pub text: String,
    pub secondary_text: String,
    pub accent: String,
    pub border: String,
    pub selection: String,
    pub warning_background: String,
    pub warning_text: String,
    pub danger_background: String,
    pub danger_text: String,
    pub success_background: String,
    pub success_text: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub edge: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub group_background: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub group_border: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub series: Option<[String; 6]>,
}

/// Why a palette was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteError {
    /// The text is not JSON, or not an object.
    Syntax(String),
    MissingField(&'static str),
    DuplicateField(&'static str),
    /// A field holds something other than a string (or, for `series`, six strings).
    WrongType(&'static str),
    /// A color is not canonical `#RRGGBB`.
    InvalidColor {
        field: String,
        value: String,
    },
}

impl fmt::Display for PaletteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaletteError::Syntax(m) => write!(f, "palette is not valid JSON: {m}"),
            PaletteError::MissingField(n) => write!(f, "missing field `{n}`"),
            PaletteError::DuplicateField(n) => write!(f, "duplicate field `{n}`"),
            PaletteError::WrongType(n) => write!(f, "field `{n}` has the wrong type"),
            PaletteError::InvalidColor { field, value } => {
                write!(f, "field `{field}` is not a canonical #RRGGBB color: {value:?}")
            }
        }
    }
}

impl std::error::Error for PaletteError {}

/// The JSON names, in field order.
pub const FIELDS: [&str; 18] = [
    "background",
    "surface",
    "surfaceSecondary",
    "text",
    "secondaryText",
    "accent",
    "border",
    "selection",
    "warningBackground",
    "warningText",
    "dangerBackground",
    "dangerText",
    "successBackground",
    "successText",
    "edge",
    "groupBackground",
    "groupBorder",
    "series",
];

const REQUIRED: usize = 14;

// serde_json refuses nesting deeper than this.
const SERDE_JSON_DEPTH: usize = 128;

fn s(v: &str) -> String {
    v.to_string()
}

impl MermaidPalette {
    pub fn light() -> Self {
        MermaidPalette {
            background: s("#FFFFFF"),
            surface: s("#F8F8F8"),
            surface_secondary: s("#F3F3F4"),
            text: s("#27272A"),
            secondary_text: s("#71717A"),
            accent: s("#2563EB"),
            border: s("#D4D4D8"),
            selection: s("#DBEAFE"),
            warning_background: s("#FEF3C7"),
            warning_text: s("#92400E"),
            danger_background: s("#FEE2E2"),
            danger_text: s("#991B1B"),
            success_background: s("#DCFCE7"),
            success_text: s("#166534"),
            edge: None,
            group_background: None,
            group_border: None,
            series: None,
        }
    }

    pub fn dark() -> Self {
        MermaidPalette {
            background: s("#18181B"),
            surface: s("#202024"),
            surface_secondary: s("#27272A"),
            text: s("#FAFAFA"),
            secondary_text: s("#A1A1AA"),
            accent: s("#60A5FA"),
            border: s("#3F3F46"),
            selection: s("#1E3A5F"),
            warning_background: s("#422006"),
            warning_text: s("#FCD34D"),
            danger_background: s("#450A0A"),
            danger_text: s("#FCA5A5"),
            success_background: s("#052E16"),
            success_text: s("#86EFAC"),
            edge: None,
            group_background: None,
            group_border: None,
            series: None,
        }
    }

    /// The neutral palette Zebra's renderer uses when no application palette is given.
    pub fn beautiful_light() -> Self {
        let mut p = Self::light();
        p.accent = mix_hex(&p.text, &p.background, 0.85);
        p
    }

    pub fn beautiful_dark() -> Self {
        let mut p = Self::dark();
        p.accent = mix_hex(&p.text, &p.background, 0.85);
        p
    }

    /// The default for a theme mode when no palette is given.
    pub fn default_for(mode: MermaidThemeMode) -> Self {
        match mode {
            MermaidThemeMode::Light => Self::beautiful_light(),
            MermaidThemeMode::Dark => Self::beautiful_dark(),
        }
    }

    /// The palette as Zebra's Swift initializer completes it: missing optional fields take the
    /// Swift defaults. This is not what the native boundary does with missing fields.
    pub fn swift_completed(&self) -> Self {
        let mut p = self.clone();
        p.edge.get_or_insert_with(|| self.secondary_text.clone());
        p.group_background.get_or_insert_with(|| self.background.clone());
        p.group_border.get_or_insert_with(|| self.border.clone());
        p.series.get_or_insert_with(|| {
            [
                self.accent.clone(),
                self.success_text.clone(),
                self.warning_text.clone(),
                self.danger_text.clone(),
                self.text.clone(),
                self.secondary_text.clone(),
            ]
        });
        p
    }

    /// The 14 required colors in field order.
    pub fn required(&self) -> [(&'static str, &str); 14] {
        [
            ("background", &self.background),
            ("surface", &self.surface),
            ("surfaceSecondary", &self.surface_secondary),
            ("text", &self.text),
            ("secondaryText", &self.secondary_text),
            ("accent", &self.accent),
            ("border", &self.border),
            ("selection", &self.selection),
            ("warningBackground", &self.warning_background),
            ("warningText", &self.warning_text),
            ("dangerBackground", &self.danger_background),
            ("dangerText", &self.danger_text),
            ("successBackground", &self.success_background),
            ("successText", &self.success_text),
        ]
    }

    /// Checks that every color is canonical `#RRGGBB`.
    pub fn validate(&self) -> Result<(), PaletteError> {
        let bad = |field: &str, value: &str| PaletteError::InvalidColor { field: field.to_string(), value: value.to_string() };
        for (name, value) in self.required() {
            if Rgb::parse(value).is_none() {
                return Err(bad(name, value));
            }
        }
        for (name, value) in [("edge", &self.edge), ("groupBackground", &self.group_background), ("groupBorder", &self.group_border)] {
            if let Some(v) = value {
                if Rgb::parse(v).is_none() {
                    return Err(bad(name, v));
                }
            }
        }
        if let Some(series) = &self.series {
            for (i, v) in series.iter().enumerate() {
                if Rgb::parse(v).is_none() {
                    return Err(bad(&format!("series[{i}]"), v));
                }
            }
        }
        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Reads palette JSON as Zebra's serde boundary does: an object with the camelCase fields
    /// (unknown fields are ignored, a repeated known field is an error, `null` leaves an
    /// optional field out), or the fields in order as an array. Colors are then validated.
    pub fn from_json(text: &str) -> Result<Self, PaletteError> {
        let palette = Self::decode_json(text)?;
        palette.validate()?;
        Ok(palette)
    }

    /// Decodes without validating the colors, as deserialization alone does.
    pub fn decode_json(text: &str) -> Result<Self, PaletteError> {
        let syntax = |e: crate::config::json::JsonError| PaletteError::Syntax(format!("{e:?}"));
        let mut values: [Option<ConfigValue>; 18] = Default::default();
        let first = text.trim_start_matches([' ', '\t', '\n', '\r']).as_bytes().first().copied();
        if first == Some(b'[') {
            let ConfigValue::Array(items) = parse_json_limited(text, SERDE_JSON_DEPTH).map_err(syntax)? else {
                return Err(PaletteError::Syntax("expected an object or an array".into()));
            };
            if items.len() < REQUIRED || items.len() > FIELDS.len() {
                return Err(PaletteError::Syntax(format!("invalid length {}", items.len())));
            }
            for (slot, item) in values.iter_mut().zip(items) {
                *slot = Some(item);
            }
        } else {
            for (key, value) in parse_json_members(text, SERDE_JSON_DEPTH).map_err(syntax)? {
                if let Some(k) = FIELDS.iter().position(|f| *f == key) {
                    if values[k].is_some() {
                        return Err(PaletteError::DuplicateField(FIELDS[k]));
                    }
                    values[k] = Some(value);
                }
            }
        }
        let required = |k: usize| -> Result<String, PaletteError> {
            match &values[k] {
                None => Err(PaletteError::MissingField(FIELDS[k])),
                Some(ConfigValue::String(v)) => Ok(v.clone()),
                Some(_) => Err(PaletteError::WrongType(FIELDS[k])),
            }
        };
        let optional = |k: usize| -> Result<Option<String>, PaletteError> {
            match &values[k] {
                None | Some(ConfigValue::Null) => Ok(None),
                Some(ConfigValue::String(v)) => Ok(Some(v.clone())),
                Some(_) => Err(PaletteError::WrongType(FIELDS[k])),
            }
        };
        let series = match &values[17] {
            None | Some(ConfigValue::Null) => None,
            Some(ConfigValue::Array(items)) if items.len() == 6 => {
                let mut out: [String; 6] = Default::default();
                for (slot, item) in out.iter_mut().zip(items) {
                    match item {
                        ConfigValue::String(v) => *slot = v.clone(),
                        _ => return Err(PaletteError::WrongType("series")),
                    }
                }
                Some(out)
            }
            Some(_) => return Err(PaletteError::WrongType("series")),
        };
        Ok(MermaidPalette {
            background: required(0)?,
            surface: required(1)?,
            surface_secondary: required(2)?,
            text: required(3)?,
            secondary_text: required(4)?,
            accent: required(5)?,
            border: required(6)?,
            selection: required(7)?,
            warning_background: required(8)?,
            warning_text: required(9)?,
            danger_background: required(10)?,
            danger_text: required(11)?,
            success_background: required(12)?,
            success_text: required(13)?,
            edge: optional(14)?,
            group_background: optional(15)?,
            group_border: optional(16)?,
            series,
        })
    }

    /// Canonical camelCase JSON: every field in order, absent optional fields as `null`, as
    /// Zebra's serde derive writes them.
    pub fn to_json(&self) -> String {
        let q = |v: &str| json_string(v);
        let opt = |v: &Option<String>| v.as_deref().map_or("null".to_string(), json_string);
        let mut out = String::from("{");
        for (name, value) in self.required() {
            out.push_str(&format!("{}:{},", q(name), q(value)));
        }
        out.push_str(&format!("\"edge\":{},", opt(&self.edge)));
        out.push_str(&format!("\"groupBackground\":{},", opt(&self.group_background)));
        out.push_str(&format!("\"groupBorder\":{},", opt(&self.group_border)));
        match &self.series {
            Some(series) => {
                let items: Vec<String> = series.iter().map(|v| q(v)).collect();
                out.push_str(&format!("\"series\":[{}]", items.join(",")));
            }
            None => out.push_str("\"series\":null"),
        }
        out.push('}');
        out
    }
}

fn json_string(v: &str) -> String {
    let mut out = String::with_capacity(v.len() + 2);
    out.push('"');
    for c in v.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = include_str!("../../../../docs/research/zebra-theme-palette.example.json");

    #[test]
    fn decodes_the_swift_shaped_example() {
        let p = MermaidPalette::from_json(EXAMPLE).unwrap();
        assert_eq!(p.background, "#FFFFFF");
        assert_eq!(p.edge.as_deref(), Some("#71717A"));
        assert_eq!(p.series.as_ref().unwrap()[0], "#2563EB");
        // Round trip through the canonical form.
        assert_eq!(MermaidPalette::from_json(&p.to_json()).unwrap(), p);
    }

    #[test]
    fn fourteen_field_json_keeps_optional_fields_absent() {
        let mut p = MermaidPalette::light();
        p.edge = None;
        let json = p.to_json().replace(",\"edge\":null,\"groupBackground\":null,\"groupBorder\":null,\"series\":null", "");
        let decoded = MermaidPalette::from_json(&json).unwrap();
        assert_eq!(decoded, MermaidPalette::light());
        assert_eq!(decoded.edge, None);
        // The Swift initializer would have filled them; the native boundary does not.
        let swift = decoded.swift_completed();
        assert_eq!(swift.edge.as_deref(), Some("#71717A"));
        assert_eq!(swift.group_background.as_deref(), Some("#FFFFFF"));
        assert_eq!(swift.group_border.as_deref(), Some("#D4D4D8"));
        assert_eq!(swift.series.as_ref().unwrap(), &["#2563EB", "#166534", "#92400E", "#991B1B", "#27272A", "#71717A"].map(String::from));
    }

    #[test]
    fn refuses_what_zebra_refuses() {
        let base = MermaidPalette::from_json(EXAMPLE).unwrap();
        let swap = |field: &str, value: &str| {
            EXAMPLE.replacen(&format!("\"{field}\": \"{}\"", value_of(&base, field)), &format!("\"{field}\": {value}"), 1)
        };
        for (field, value) in [
            ("accent", "\"#2563eb\""),
            ("accent", "\"#FFF\""),
            ("text", "\"rgba(0,0,0,.2)\""),
            ("edge", "\"var(--x)\""),
            ("border", "\"#2563EBFF\""),
        ] {
            let json = swap(field, value);
            assert!(matches!(MermaidPalette::from_json(&json), Err(PaletteError::InvalidColor { .. })), "{field} {value}");
        }
        assert_eq!(MermaidPalette::from_json(&swap("text", "null")), Err(PaletteError::WrongType("text")));
        assert_eq!(MermaidPalette::from_json(&swap("accent", "5")), Err(PaletteError::WrongType("accent")));
        let missing = EXAMPLE.replace("\"selection\": \"#DBEAFE\",", "");
        assert_eq!(MermaidPalette::from_json(&missing), Err(PaletteError::MissingField("selection")));
        let five = EXAMPLE.replace(", \"#71717A\"]", "]");
        assert_eq!(MermaidPalette::from_json(&five), Err(PaletteError::WrongType("series")));
        let duplicate = EXAMPLE.replacen("{", "{\n  \"text\": \"#000000\",", 1);
        assert_eq!(MermaidPalette::from_json(&duplicate), Err(PaletteError::DuplicateField("text")));
        assert!(matches!(MermaidPalette::from_json("[1, 2]"), Err(PaletteError::Syntax(_))));
        assert!(matches!(MermaidPalette::from_json("{"), Err(PaletteError::Syntax(_))));
        // Unknown fields are ignored, as serde does by default.
        let extra = EXAMPLE.replacen("{", "{\n  \"futureRole\": {\"nested\": [1, 2]},", 1);
        assert_eq!(MermaidPalette::from_json(&extra).unwrap(), base);
        // A null optional field is absent.
        let null_edge = EXAMPLE.replace("\"edge\": \"#71717A\"", "\"edge\": null");
        assert_eq!(MermaidPalette::from_json(&null_edge).unwrap().edge, None);
    }

    fn value_of(p: &MermaidPalette, field: &str) -> String {
        match field {
            "accent" => p.accent.clone(),
            "text" => p.text.clone(),
            "border" => p.border.clone(),
            "edge" => p.edge.clone().unwrap(),
            other => panic!("{other}"),
        }
    }

    #[test]
    fn defaults_follow_zebra() {
        assert_eq!(MermaidPalette::beautiful_light().accent, mix_hex("#27272A", "#FFFFFF", 0.85));
        assert_eq!(MermaidPalette::default_for(MermaidThemeMode::Dark).background, "#18181B");
        assert!(MermaidPalette::beautiful_dark().is_valid());
    }
}
