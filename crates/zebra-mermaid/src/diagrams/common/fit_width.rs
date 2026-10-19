//! Fitting a drawing into the width the host has. Ported from Pele
//! `src/diagrams/common/fit-width.ts`.

use crate::diagrams::Rendered;
use crate::error::Error;
use crate::options::RenderOptions;

/// Draws a chart whose plot can be any width so that the whole diagram fits the width the
/// host has. What surrounds the plot (labels, a legend) is only known once it is drawn, so the
/// plot gives up the overflow and is drawn again. `narrow` tells the chart that it did not fit
/// at its own width, for a chart that then arranges itself differently.
pub fn fit_width(
    options: &RenderOptions,
    natural: f64,
    least: f64,
    mut draw: impl FnMut(f64, bool) -> Result<Rendered, Error>,
) -> Result<Rendered, Error> {
    let mut result = draw(natural, false)?;
    let available = match options.max_width {
        Some(a) if a > 0.0 => a,
        _ => return Ok(result),
    };
    if result.width <= available {
        return Ok(result);
    }
    let floor = crate::util::js::min(least, natural);
    let mut plot = natural;
    result = draw(plot, true)?;
    let mut round = 0;
    while round < 3 && result.width > available && plot > floor {
        plot = crate::util::js::max(floor, plot - (result.width - available));
        result = draw(plot, true)?;
        round += 1;
    }
    Ok(result)
}

/// The width a title may take before it wraps.
pub fn title_room(options: &RenderOptions) -> f64 {
    match options.max_width {
        Some(a) if a > 0.0 => crate::util::js::max(80.0, a - 2.0 * options.padding.unwrap_or(8.0)),
        _ => 4000.0,
    }
}

/// For a chart that is drawn to a set width: its own, or the host's when that is smaller.
/// `around` is what the chart adds outside that width.
pub fn cap_width(options: &RenderOptions, natural: f64, least: f64, around: f64) -> f64 {
    match options.max_width {
        Some(a) if a > 0.0 => crate::util::js::max(crate::util::js::min(least, natural), crate::util::js::min(natural, a - around)),
        _ => natural,
    }
}

const BREAKPOINT: f64 = 640.0;

/// For a diagram that can run across or down: draws it across, and down instead when the host
/// allows it, the width it has is under the breakpoint, across does not fit, and down is
/// narrower.
pub fn turn_to_fit(options: &RenderOptions, mut draw: impl FnMut(bool) -> Result<Rendered, Error>) -> Result<Rendered, Error> {
    let across = draw(false)?;
    let available = match options.max_width {
        Some(a) if a > 0.0 => a,
        _ => return Ok(across),
    };
    if options.auto_direction == Some(false) || across.width <= available {
        return Ok(across);
    }
    let breakpoint = match options.direction_breakpoint {
        Some(b) if b > 0.0 => b,
        _ => BREAKPOINT,
    };
    if available >= breakpoint {
        return Ok(across);
    }
    let down = draw(true)?;
    Ok(if down.width < across.width { down } else { across })
}

pub fn runs_across(direction: Option<&str>) -> bool {
    matches!(direction, Some("LR" | "RL"))
}

/// Draws a diagram with room to spare, and tightly instead when that does not fit the width
/// the host has and tight is narrower.
pub fn tighten(options: &RenderOptions, mut draw: impl FnMut(bool) -> Result<Rendered, Error>) -> Result<Rendered, Error> {
    let loose = draw(false)?;
    let available = match options.max_width {
        Some(a) if a > 0.0 => a,
        _ => return Ok(loose),
    };
    if loose.width <= available {
        return Ok(loose);
    }
    let tight = draw(true)?;
    Ok(if tight.width < loose.width { tight } else { loose })
}
