//! The info diagram. Ported from Pele `src/diagrams/info/`.

use crate::diagrams::common::parse_tokens::Grammar;
use crate::svg::concat::svg_format;
use std::sync::LazyLock;

use crate::config::Config;
use crate::detect::DiagramType;
use crate::diagrams::common::tokens::{
    acc_descr_value, acc_title_value, title_value, TokenType, ACC_DESCR, ACC_TITLE, DIRECTIVE, EOF, NEWLINE, SINGLE_LINE_COMMENT, TITLE,
    WHITESPACE, YAML,
};
use crate::diagrams::{ParseInput, RenderCtx, Rendered};
use crate::error::Error;
use crate::keyword_token;
use crate::svg::builder::render_label_svg as label_svg;
use crate::svg::root::{svg_document, Accessible};
use crate::text::{layout_render_label as layout_label, style};
use crate::theme::Role;
use crate::util::js;

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct InfoModel {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
}

const SHOW_INFO: usize = 0;
const INFO: usize = 1;
const T_ACC_DESCR: usize = 2;
const T_ACC_TITLE: usize = 3;
const T_TITLE: usize = 4;
const T_NEWLINE: usize = 5;

pub static INFO_TOKENS: LazyLock<Vec<TokenType>> = LazyLock::new(|| {
    vec![
        keyword_token!("showInfo"),
        keyword_token!("info"),
        ACC_DESCR,
        ACC_TITLE,
        TITLE,
        NEWLINE,
        WHITESPACE,
        YAML,
        DIRECTIVE,
        SINGLE_LINE_COMMENT,
    ]
});

#[derive(Debug, Default)]
pub struct InfoAst {
    pub title: Option<String>,
    pub acc_title: Option<String>,
    pub acc_descr: Option<String>,
}

pub fn parse_info(src: &str) -> Result<InfoAst, Error> {
    static GRAMMAR: LazyLock<Grammar> = LazyLock::new(|| Grammar::new(&INFO_TOKENS));
    let mut r = GRAMMAR.reader(src, DiagramType::Info)?;
    let mut ast = InfoAst::default();
    while r.kind() == T_NEWLINE {
        r.i += 1;
    }
    r.expect(INFO)?;
    while r.kind() == T_NEWLINE {
        r.i += 1;
    }
    if r.accept(SHOW_INFO) {
        while r.kind() == T_NEWLINE {
            r.i += 1;
        }
    }
    while r.kind() != EOF {
        match r.kind() {
            T_ACC_DESCR => ast.acc_descr = Some(acc_descr_value(r.take())),
            T_ACC_TITLE => ast.acc_title = Some(acc_title_value(r.take())),
            T_TITLE => ast.title = Some(title_value(r.take())),
            _ => return Err(r.fail("a title or the end of input")),
        }
        if r.kind() == EOF {
            break;
        }
        if r.kind() != T_NEWLINE {
            return Err(r.fail("a line break or the end of input"));
        }
        while r.kind() == T_NEWLINE {
            r.i += 1;
        }
    }
    Ok(ast)
}

pub fn build_info(ast: InfoAst, title: Option<&str>) -> InfoModel {
    InfoModel {
        title: ast.title.filter(|t| !t.is_empty()).or_else(|| title.map(str::to_string)),
        acc_title: ast.acc_title.filter(|t| !t.is_empty()).map(|t| js::trim_start(&t).to_string()),
        acc_descr: ast.acc_descr.filter(|t| !t.is_empty()).map(|t| crate::diagrams::squeeze_line_starts(&t)),
    }
}

pub(crate) fn parse(input: &ParseInput) -> Result<InfoModel, Error> {
    Ok(build_info(parse_info(input.source)?, input.title))
}

/// The text the diagram shows by default.
pub fn default_text() -> String {
    // Preserve the fixed reference's observable output, including its version diagram.
    // Hosts can set info_text to expose their own product version instead.
    format!("Pele v{}", crate::PELE_VERSION)
}

pub(crate) fn render(model: &InfoModel, _config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    let options = ctx.options;
    let size = options.font_size.unwrap_or(16.0);
    let m = ctx.measure();
    let pad = options.padding.unwrap_or(8.0);

    let title = layout_label(model.title.as_deref(), false, &m.without_math(), size, 4000.0, style::BOLD);
    let title_height = if title.height > 0.0 { title.height + 8.0 } else { 0.0 };
    let shown = options.info_text.clone().unwrap_or_else(default_text);
    let text = layout_label(Some(&shown), false, &m, size, 4000.0, 0);

    let width = (js::max(title.width, text.width) + 2.0 * pad).ceil();
    let height = (title_height + text.height + 2.0 * pad).ceil();
    let content = svg_format!(
        label_svg(&title, width / 2.0, pad + title.height / 2.0, " class=\"pele-title\" font-weight=\"var(--_tw)\"", None),
        label_svg(
            &text,
            width / 2.0,
            pad + title_height + text.height / 2.0,
            &svg_format!(" class=\"pele-version\" fill=\"", ctx.paint(Role::SecondaryText, "var(--_m)"), "\""),
            None,
        )
    );
    let acc = Accessible { acc_title: model.acc_title.as_deref(), acc_descr: model.acc_descr.as_deref() };
    Ok(Rendered { svg: svg_document("info", width, height, size, ctx, acc, &content), width, height, links: Vec::new() })
}

#[cfg(test)]
mod compact_token_reference {
    #[test]
    fn pinned_node_token_streams_and_errors() {
        crate::diagrams::common::parse_tokens::tests::check_table("info", &super::INFO_TOKENS, crate::DiagramType::Info);
    }
}
