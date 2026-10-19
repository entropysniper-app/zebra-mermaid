//! Swimlanes share flowchart syntax, with the lane layout policy.
pub type SwimlaneModel = super::flowchart::FlowchartModel;
use super::{ParseInput, RenderCtx, Rendered};
use crate::{Config, Error};
pub(crate) fn parse(input: &ParseInput) -> Result<SwimlaneModel, Error> {
    super::flowchart::parse(input)
}
pub(crate) fn render(model: &SwimlaneModel, config: &Config, ctx: &RenderCtx) -> Result<Rendered, Error> {
    super::flowchart::render_variant(model, config, ctx, true)
}
