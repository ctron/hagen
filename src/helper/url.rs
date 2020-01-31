use crate::generator;
use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
    Renderable,
};
use url::Url;

use crate::generator::GeneratorContext;
use failure::Error;
use log::debug;

use std::sync::{Arc, RwLock};

pub fn full_url_for<S: AsRef<str>>(basename: &Url, path: S) -> Result<Url, Error> {
    let path = path.as_ref();

    // if we have an absolute URL, then absolute is still relative to the site base
    let path = if path.starts_with('/') {
        &path[1..]
    } else {
        path
    };

    Ok(basename.join(path)?)
}

pub fn full_url_from(url: &str, output: &generator::Output) -> Result<url::Url, RenderError> {
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

fn full_url<'reg: 'rc, 'rc>(
    h: &Helper<'reg, 'rc>,
    output: &generator::Output,
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

    full_url_from(url, output)
}

pub struct AbsoluteUrlHelper {
    pub context: Arc<RwLock<Option<GeneratorContext>>>,
}

impl HelperDef for AbsoluteUrlHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let context = self.context.read();
        let context = context
            .as_ref()
            .map_err(|_| RenderError::new("Failed to get generator context"))?
            .as_ref()
            .unwrap();
        let url = full_url(h, &context.output)?;

        out.write(url.as_str())?;

        Ok(())
    }
}

pub struct RelativeUrlHelper {
    pub context: Arc<RwLock<Option<GeneratorContext>>>,
}

impl HelperDef for RelativeUrlHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let context = self.context.read();
        let context = context
            .as_ref()
            .map_err(|_| RenderError::new("Failed to get generator context"))?
            .as_ref()
            .unwrap();
        let url = full_url(h, &context.output)?;

        out.write(url.path())?;

        Ok(())
    }
}

pub struct ActiveHelper {
    pub context: Arc<RwLock<Option<GeneratorContext>>>,
}

impl HelperDef for ActiveHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
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

        let context = self.context.read();
        let context = context
            .as_ref()
            .map_err(|_| RenderError::new("Failed to get generator context"))?
            .as_ref()
            .unwrap();
        let check_url = full_url_from(&url, &context.output)?;
        let page_url = full_url_from("", &context.output)?;

        debug!("check: {} - page: {}", check_url, page_url);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::Output;
    use std::str::FromStr;

    fn test_full_url(site_url: &str, path: &str, url: &str, expected: &str) {
        let o = Output {
            site_url: site_url.into(),
            path: path.into(),
            template: None,
        };
        assert_eq!(
            full_url_from(url, &o).expect(""),
            Url::from_str(expected).expect("")
        );
    }

    #[test]
    fn test_1() {
        test_full_url(
            "http://localhost:8080/",
            "index.html",
            "",
            "http://localhost:8080/index.html",
        );
    }

    #[test]
    fn test_2() {
        test_full_url(
            "http://localhost:8080/site/",
            "index.html",
            "",
            "http://localhost:8080/site/index.html",
        );
    }

    #[test]
    fn test_3() {
        test_full_url(
            "http://localhost:8080/site/",
            "index.html",
            "/",
            "http://localhost:8080/site/",
        );
    }
}
