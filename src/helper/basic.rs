use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
    Renderable, TemplateRenderError,
};

use log::{debug, info};

#[derive(Clone, Copy)]
pub struct TimesHelper;

impl HelperDef for TimesHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let n = h
            .param(0)
            .ok_or(RenderError::new("Missing parameter for times"))?;

        if let Some(body) = h.template() {
            match n.value().as_u64() {
                Some(n) => {
                    info!("Repeating {} times", n);
                    for _ in 0..n {
                        body.render(r, ctx, rc, out)?;
                    }
                }
                _ => Err(RenderError::new("Unable to parse parameter as number"))?,
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct ExpandHelper;

impl HelperDef for ExpandHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        hb: &Handlebars,
        ctx: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let value = h.param(0).ok_or(RenderError::new("Missing value"))?;

        let template = value
            .value()
            .as_str()
            .ok_or(RenderError::new("Unable to get template data as string"))?;

        let result = hb.render_template(template, ctx.data()).map_err(|e| {
            RenderError::new(format!("Failed to process template: {}", e.to_string()))
        })?;

        out.write(&result)?;

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct RelativeUrlHelper;

impl HelperDef for RelativeUrlHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let url = h
            .param(0)
            .ok_or(RenderError::new("Missing URL parameter for relative_url"))?
            .value()
            .as_str()
            .ok_or(RenderError::new("Wrong value type of URL. Must be string."))?;

        out.write(url)?;

        Ok(())
    }
}
