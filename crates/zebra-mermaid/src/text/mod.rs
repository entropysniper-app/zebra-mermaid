//! Text: entity codes, width measurement, icons and label layout.

pub mod entities;
pub mod icons;
pub mod label;
pub mod math;
pub mod measurer;
mod metrics;
pub(crate) mod render_label;
pub(crate) use label::layout_render_label;
pub(crate) use render_label::RenderLabel;

use std::rc::Rc;

pub use icons::{IconResolver, IconTable, Icons};
pub use label::{layout_label, Label, Span, LINE_HEIGHT};
pub use math::MathMarkup;
pub use measurer::{style, MetricsMeasurer, TextMeasurer};

/// What label layout measures with during one render: the host's measurer and the icons the
/// host has. Pele keeps the icons in module state; here they travel with the measurer.
#[derive(Clone, Copy)]
pub struct Measure<'a> {
    measurer: &'a dyn TextMeasurer,
    icons: &'a Icons,
    math: Option<&'a math::Math>,
    math_enabled: bool,
}

impl<'a> Measure<'a> {
    pub fn new(measurer: &'a dyn TextMeasurer, icons: &'a Icons) -> Self {
        Measure { measurer, icons, math: None, math_enabled: true }
    }

    pub(crate) fn with_math(measurer: &'a dyn TextMeasurer, icons: &'a Icons, math: Option<&'a math::Math>) -> Self {
        Self { measurer, icons, math, math_enabled: math.is_some() }
    }

    pub fn without_math(self) -> Self {
        Self { math: None, math_enabled: false, ..self }
    }

    pub(crate) fn math_active(&self) -> bool {
        self.math_enabled
    }

    pub(crate) fn typeset(&self, tex: &str, size: f64) -> Option<Rc<MathMarkup>> {
        if !self.math_enabled {
            return None;
        }
        self.math.map_or_else(|| math::typeset(tex, size).map(Rc::new), |math| math.typeset(tex, size))
    }

    #[inline]
    pub fn width(&self, text: &str, size: f64, style: u8) -> f64 {
        self.measurer.width(text, size, style)
    }

    /// The host's markup for an icon, or an empty string when it has none.
    #[inline]
    pub fn icon_markup(&self, name: &str) -> Rc<str> {
        self.icons.markup(name)
    }

    #[inline]
    pub fn icons(&self) -> &'a Icons {
        self.icons
    }

    #[inline]
    pub fn measurer(&self) -> &'a dyn TextMeasurer {
        self.measurer
    }
}
