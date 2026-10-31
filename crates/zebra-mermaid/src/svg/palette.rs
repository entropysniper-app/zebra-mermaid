//! Per-render Mermaid palette variables, from Pele 1.13.0 `src/svg/palette.ts` (MIT).
use super::{
    builder::esc,
    ink::{ink, plain_color},
};
use crate::{config::Config, detect::DiagramType, util::js};
#[derive(Default)]
pub struct Palette {
    pub style: String,
    colors: [Option<String>; 8],
    inks: [Option<&'static str>; 8],
}
fn split_list(value: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0);
    for (i, c) in value.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(js::trim(&value[start..i]));
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(js::trim(&value[start..]));
    out
}
fn safe_color(value: &str) -> Option<&str> {
    let color = js::trim(value);
    if color.is_empty()
        || js::len16(color) > 200
        || !color.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || crate::util::chars::is_space(c) || "#%.,+-/()".contains(c))
    {
        return None;
    }
    if !color.contains('(') {
        return plain_color(color).then_some(color);
    }
    let mut depth = 0;
    for (i, c) in color.char_indices() {
        if c == '(' {
            depth += 1;
            let before = color[..i].trim_end_matches(crate::util::chars::is_space);
            let from = before
                .char_indices()
                .rev()
                .find(|(_, c)| !(c.is_ascii_alphanumeric() || *c == '_' || *c == '-'))
                .map_or(0, |(p, c)| p + c.len_utf8());
            let name = before[from..].to_ascii_lowercase();
            if !["rgb", "rgba", "hsl", "hsla", "hwb", "lab", "lch", "oklab", "oklch", "color", "color-mix", "light-dark", "var"]
                .contains(&name.as_str())
            {
                return None;
            }
        } else if c == ')' {
            depth -= 1;
            if depth < 0 {
                return None;
            }
        }
    }
    if depth != 0 {
        return None;
    }
    let lower = color.to_ascii_lowercase();
    let mut from = 0;
    while let Some(at) = lower[from..].find("var(") {
        let body = js::trim_start(&color[from + at + 4..]);
        let name = body.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-')).next().unwrap_or("");
        if !name.starts_with("--pele-") {
            return None;
        }
        from += at + 4;
    }
    Some(color)
}
impl Palette {
    pub fn new(kind: DiagramType, config: &Config) -> Self {
        let mut result = Self::default();
        let Some(vars) = config.section("themeVariables") else { return result };
        let (values, repeat): (Vec<Option<&str>>, bool) = if kind == DiagramType::Xychart {
            let Some(s) = vars.section("xyChart").and_then(|v| v.get("plotColorPalette")).and_then(|v| v.as_str()) else { return result };
            (split_list(s).into_iter().map(Some).collect(), true)
        } else {
            let (prefix, first, repeat) = match kind {
                DiagramType::Pie => ("pie", 1, false),
                DiagramType::GitGraph => ("git", 0, false),
                DiagramType::Timeline | DiagramType::Radar | DiagramType::Treemap => ("cScale", 0, false),
                DiagramType::Mindmap => ("cScale", 1, false),
                DiagramType::Venn => ("venn", 1, true),
                _ => return result,
            };
            let mut list = Vec::new();
            for i in 0..8 {
                let value = vars.get(&format!("{prefix}{}", first + i));
                if repeat && value.is_none_or(|v| v.is_null() || v.as_str() == Some("")) {
                    continue;
                }
                list.push(value.and_then(|v| v.as_str()));
            }
            (list, repeat)
        };
        if values.is_empty() {
            return result;
        }
        for i in 0..8 {
            if !repeat && i >= values.len() {
                break;
            }
            let Some(color) = values[i % values.len()].and_then(safe_color) else { continue };
            result.style += &format!(";--pele-series-{}:{}", i + 1, color);
            result.inks[i] = ink(&format!(";fill:{color}"), "");
            result.colors[i] = Some(color.into());
        }
        result.style = esc(&result.style).into_owned();
        result
    }
    pub fn ink_fill(&self, index: i64) -> String {
        self.inks[index.rem_euclid(8) as usize].map_or_else(String::new, |s| format!(" fill=\"{s}\""))
    }
    pub fn native(&self, index: i64, theming: &crate::theme::Theming) -> Option<String> {
        fn resolve(raw: &str, t: &crate::theme::Theming, depth: usize) -> Option<String> {
            if depth > 8 {
                return None;
            }
            let s = js::trim(raw);
            let lower = s.to_ascii_lowercase();
            if lower == "currentcolor" {
                return Some(t.resolved.text.clone());
            }
            if plain_color(s) {
                return Some(s.into());
            }
            if lower.starts_with("var(") && s.ends_with(')') {
                let args = split_list(&s[4..s.len() - 1]);
                let c = match args[0] {
                    "--pele-accent" => Some(&t.resolved.accent),
                    "--pele-bg" => Some(&t.resolved.background),
                    "--pele-fg" => Some(&t.resolved.text),
                    "--pele-muted" => Some(&t.resolved.secondary_text),
                    "--pele-border" => Some(&t.resolved.node_stroke),
                    _ => None,
                };
                return c.cloned().or_else(|| args.get(1).and_then(|s| resolve(s, t, depth + 1)));
            }
            if lower.starts_with("light-dark(") && s.ends_with(')') {
                let args = split_list(&s[11..s.len() - 1]);
                if args.len() == 2 {
                    return resolve(args[usize::from(t.theme_mode == crate::MermaidThemeMode::Dark)], t, depth + 1);
                }
            }
            if ["rgb(", "rgba(", "hsl(", "hsla("].iter().any(|p| lower.starts_with(p)) && !lower.contains("var(") {
                return Some(s.into());
            }
            None
        }
        self.colors[index.rem_euclid(8) as usize].as_deref().and_then(|s| resolve(s, theming, 0))
    }
}
