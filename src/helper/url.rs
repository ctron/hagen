use crate::generator;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
};
use url::Url;

use log::info;

fn full_url<'reg: 'rc, 'rc>(
    h: &Helper<'reg, 'rc>,
    ctx: &'rc Context,
) -> Result<url::Url, RenderError> {
    let url = h
        .param(0)
        .ok_or(RenderError::new("Missing URL parameter for absolute_url"))?
        .value()
        .as_str()
        .ok_or(RenderError::new("Wrong value type of URL. Must be string."))?;

    let output = generator::Output::from(ctx)?;

    // start with the site base name
    let result = Url::parse(&output.site_url).map_err(|err| RenderError::with(err))?;

    info!("URL1: {:?}", result);

    // if we have an absolute URL, then absolute is still relative to the site base
    let result = if !url.starts_with("/") {
        // the url is relative to the page, not the site
        result
            .join(&output.path)
            .map_err(|err| RenderError::with(err))?
            .join(url)
            .map_err(|err| RenderError::with(err))?
    } else {
        let url = &url[1..];
        result.join(url).map_err(|err| RenderError::with(err))?
    };

    // append the url
    let result = result;

    info!("URL3: {:?}", result);

    Ok(result)
}

#[derive(Clone, Copy)]
pub struct AbsoluteUrlHelper;

impl HelperDef for AbsoluteUrlHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars,
        ctx: &'rc Context,
        _: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let url = full_url(h, ctx)?;

        out.write(url.as_str())?;

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct RelativeUrlHelper;

impl HelperDef for RelativeUrlHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars,
        ctx: &'rc Context,
        _: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let url = full_url(h, ctx)?;

        out.write(url.path())?;

        Ok(())
    }
}
