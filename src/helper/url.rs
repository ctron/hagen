use crate::generator;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
    Renderable,
};
use url::Url;

use log::info;

fn full_url_from(url: &str, ctx: &Context) -> Result<url::Url, RenderError> {
    let output = generator::Output::from(ctx)?;

    // start with the site base name
    let result = Url::parse(&output.site_url).map_err(|err| RenderError::with(err))?;

    // if we have an absolute URL, then absolute is still relative to the site base
    let result = if url.is_empty() {
        result
            .join(&output.path)
            .map_err(|err| RenderError::with(err))?
    } else if !url.starts_with("/") {
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

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::Output;
    use serde_json::Map;
    use std::str::FromStr;

    #[test]
    fn test_1() {
        let o = Output {
            site_url: "http://localhost:8080/".into(),
            path: "index.html".into(),
        };
        let mut m = Map::new();
        m.insert("output".into(), serde_json::to_value(o).expect(""));
        let ctx = Context::wraps(m).expect("");
        assert_eq!(
            full_url_from("", &ctx).expect(""),
            Url::from_str("http://localhost:8080/").expect("")
        );
    }
}

fn full_url<'reg: 'rc, 'rc>(
    h: &Helper<'reg, 'rc>,
    ctx: &'rc Context,
) -> Result<url::Url, RenderError> {
    let url = h
        .param(0)
        .ok_or(RenderError::new(format!(
            "Missing URL parameter for {}",
            h.name()
        )))?
        .value()
        .as_str()
        .ok_or(RenderError::new("Wrong value type of URL. Must be string."))?;

    full_url_from(url, ctx)
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

#[derive(Clone, Copy)]
pub struct ActiveHelper;

impl HelperDef for ActiveHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let mut url = h
            .param(0)
            .ok_or(RenderError::new("Missing URL parameter for 'active'"))?
            .value()
            .as_str()
            .map(|s| String::from(s))
            .ok_or(RenderError::new("Wrong value type of URL. Must be string."))?;

        if url.ends_with("/") {
            url.push_str("index.html")
        }

        let check_url = full_url_from(&url, ctx)?;
        let page_url = full_url_from("", ctx)?;

        info!("check: {} - page: {}", check_url, page_url);

        if check_url == page_url {
            if let Some(t) = h.template() {
                t.render(r, ctx, rc, out)?;
            } else {
                let value = h
                    .param(1)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("active");
                out.write(value)?;
            }
        }

        Ok(())
    }
}
