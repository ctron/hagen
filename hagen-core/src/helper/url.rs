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
    let url = url_param(&h)?;
    full_url_from(&url, output)
}

fn url_param(h: &Helper) -> Result<String, RenderError> {
    Ok(h.param(0)
        .ok_or(RenderError::new(format!(
            "Missing URL parameter for {}",
            h.name()
        )))?
        .value()
        .as_str()
        .ok_or(RenderError::new("Wrong value type of URL. Must be string."))?
        .into())
}

pub struct AbsoluteUrlHelper {
    pub context: Arc<RwLock<Option<GeneratorContext>>>,
}

use log::info;

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
        // early check if this is a full url

        let url = url_param(h)?;
        info!("Url: {:?}", url);
        if let Ok(url) = Url::parse(&url) {
            out.write(url.as_str())?;
            return Ok(());
        }

        // otherwise build up from relative parts

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
    use crate::generator::{GeneratorConfig, Output};
    use serde_json::Map;
    use std::str::FromStr;

    fn test_full_url(site_url: &str, path: &str, url: &str, expected: &str) {
        let o = Output {
            site_url: site_url.into(),
            url: url.to_string(),
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

    fn setup(h: &mut Handlebars) -> Result<(), Error> {
        setup_with(h, "http://localhost/base/", "/foo/bar")
    }

    fn setup_with(h: &mut Handlebars, base: &str, path: &str) -> Result<(), Error> {
        let context_provider = Arc::new(RwLock::new(None));

        h.register_helper(
            "relative_url",
            Box::new(RelativeUrlHelper {
                context: context_provider.clone(),
            }),
        );
        h.register_helper(
            "absolute_url",
            Box::new(AbsoluteUrlHelper {
                context: context_provider.clone(),
            }),
        );
        h.register_helper(
            "active",
            Box::new(ActiveHelper {
                context: context_provider.clone(),
            }),
        );

        let config = GeneratorConfig {
            basename: Url::parse(base)?,
            root: "/tmp".into(),
            output: "/tmp/output".into(),
        };

        let output = Output::new(config.basename.to_string(), path, Option::None::<String>)?;
        let ctx = GeneratorContext::new(&config, &output);
        *context_provider.write().unwrap() = Some(ctx);

        Ok(())
    }

    #[test]
    fn test_relative_url_1() -> Result<(), Error> {
        let data = Map::new();
        let mut h = Handlebars::new();

        setup(&mut h)?;

        assert_eq!(
            h.render_template(r#"{{ relative_url "https://foo.bar/baz" }}"#, &data)?,
            "https://foo.bar/baz",
        );
        Ok(())
    }

    #[test]
    fn test_relative_url_2() -> Result<(), Error> {
        let data = Map::new();
        let mut h = Handlebars::new();

        setup(&mut h)?;

        assert_eq!(
            h.render_template(r#"{{ relative_url "/baz/buz/boz" }}"#, &data)?,
            "/base/baz/buz/boz",
        );
        Ok(())
    }

    #[test]
    fn test_relative_url_3() -> Result<(), Error> {
        let data = Map::new();
        let mut h = Handlebars::new();

        setup(&mut h)?;

        assert_eq!(
            h.render_template(r#"{{ relative_url "boz" }}"#, &data)?,
            "/base/foo/boz",
        );
        Ok(())
    }

    #[test]
    fn test_absolute_url_1() -> Result<(), Error> {
        let data = Map::new();
        let mut h = Handlebars::new();

        setup(&mut h)?;

        assert_eq!(
            h.render_template(r#"{{ absolute_url "https://foo.bar/baz" }}"#, &data)?,
            "https://foo.bar/baz",
        );
        Ok(())
    }

    #[test]
    fn test_absolute_url_2() -> Result<(), Error> {
        let data = Map::new();
        let mut h = Handlebars::new();

        setup(&mut h)?;

        assert_eq!(
            h.render_template(r#"{{ absolute_url "/baz/buz/boz" }}"#, &data)?,
            "http://localhost/base/baz/buz/boz",
        );
        Ok(())
    }

    #[test]
    fn test_absolute_url_3() -> Result<(), Error> {
        let data = Map::new();
        let mut h = Handlebars::new();

        setup(&mut h)?;

        assert_eq!(
            h.render_template(r#"{{ absolute_url "boz" }}"#, &data)?,
            "http://localhost/base/foo/boz",
        );
        Ok(())
    }

    #[test]
    fn test_active_1() -> Result<(), Error> {
        let data = Map::new();
        let mut h = Handlebars::new();

        setup_with(&mut h, "http://localhost/base", "/root")?;

        assert_eq!(
            h.render_template(r#"{{ active "/root" }}"#, &data)?,
            "active",
        );
        assert_eq!(h.render_template(r#"{{ active "/root/" }}"#, &data)?, "",);
        assert_eq!(h.render_template(r#"{{ active "/" }}"#, &data)?, "",);
        assert_eq!(h.render_template(r#"{{ active "/root/bar" }}"#, &data)?, "",);

        Ok(())
    }

    #[test]
    fn test_active_2() -> Result<(), Error> {
        let data = Map::new();
        let mut h = Handlebars::new();

        setup_with(&mut h, "http://localhost/base", "/root/index.html")?;

        assert_eq!(h.render_template(r#"{{ active "/root" }}"#, &data)?, "",);
        assert_eq!(
            h.render_template(r#"{{ active "/root/" }}"#, &data)?,
            "active",
        );
        assert_eq!(h.render_template(r#"{{ active "/" }}"#, &data)?, "",);
        assert_eq!(h.render_template(r#"{{ active "/root/bar" }}"#, &data)?, "",);

        Ok(())
    }

    #[test]
    fn test_active_3() -> Result<(), Error> {
        let data = Map::new();
        let mut h = Handlebars::new();

        setup_with(&mut h, "http://localhost/base", "/root/bar.html")?;

        assert_eq!(h.render_template(r#"{{ active "/root" }}"#, &data)?, "",);
        assert_eq!(h.render_template(r#"{{ active "/root/" }}"#, &data)?, "",);
        assert_eq!(
            h.render_template(r#"{{ active "/root/bar.html" }}"#, &data)?,
            "active",
        );
        assert_eq!(h.render_template(r#"{{ active "/" }}"#, &data)?, "",);
        assert_eq!(h.render_template(r#"{{ active "/root/bar" }}"#, &data)?, "",);

        Ok(())
    }
}
