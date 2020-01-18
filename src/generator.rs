use std::fs;
use std::path::{Path, PathBuf};

use handlebars::{Context, Handlebars};

use log::{debug, info};

use crate::error::GeneratorError;
use crate::loader::directory::DirectoryLoader;
use crate::loader::Loader;
use crate::rules::{Asset, Render, Rule};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use std::fs::File;

type Result<T> = std::result::Result<T, GeneratorError>;

use crate::helper::basic::{ConcatHelper, ExpandHelper, TimesHelper};
use crate::helper::markdown::MarkdownifyHelper;

use crate::copy;
use crate::helper::time::TimeHelper;
use crate::helper::url::{AbsoluteUrlHelper, ActiveHelper, RelativeUrlHelper};
use relative_path::RelativePath;

use crate::helper::sort::SortedHelper;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE: Regex = Regex::new(r"/{2,}").unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    // path of the output file
    pub path: String,
    // the site base name
    pub site_url: String,
}

impl Output {
    pub fn new<S1, S2>(site_url: S1, path: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Output {
            path: path.into(),
            site_url: site_url.into(),
        }
    }

    pub fn from(ctx: &Context) -> Result<Self> {
        let output = ctx.data().as_object().ok_or(GeneratorError::Error(
            "'output' variable is missing or not an object".into(),
        ))?;
        let output = output
            .get(&"output".to_string())
            .ok_or(GeneratorError::Error(
                "'output' variable is missing or not an object".into(),
            ))?;

        Ok(serde_json::from_value(output.clone())?)
    }
}

pub struct Generator<'a> {
    site_url: String,

    root: PathBuf,
    handlebars: Handlebars<'a>,

    full_content: Value,
    compact_content: Value,
}

impl Generator<'_> {
    fn output(&self) -> PathBuf {
        self.root.join("output")
    }

    pub fn new<P, S>(root: P, site_url: S) -> Self
    where
        P: AsRef<Path>,
        S: Into<String>,
    {
        // create instance

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        // register helpers

        handlebars.register_helper("times", Box::new(TimesHelper));
        handlebars.register_helper("expand", Box::new(ExpandHelper));
        handlebars.register_helper("concat", Box::new(ConcatHelper));

        handlebars.register_helper("sorted", Box::new(SortedHelper));

        handlebars.register_helper("absolute_url", Box::new(AbsoluteUrlHelper));
        handlebars.register_helper("relative_url", Box::new(RelativeUrlHelper));
        handlebars.register_helper("active", Box::new(ActiveHelper));

        handlebars.register_helper("markdownify", Box::new(MarkdownifyHelper));

        handlebars.register_helper("timestamp", Box::new(TimeHelper));

        // create generator

        Generator {
            site_url: site_url.into(),
            root: root.as_ref().to_path_buf(),
            handlebars,
            full_content: Default::default(),
            compact_content: Default::default(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        debug!("Running generator");

        self.handlebars
            .register_templates_directory(".hbs", self.root.join("templates"))?;

        // clean output
        self.clean()?;

        // load data
        self.load_content()?;

        // load rules
        info!("Loading render rules");
        let render = Render::load_from(self.root.join("render.yaml"))?;

        self.build(&render)?;

        // render pages

        Ok(())
    }

    fn load_content(&mut self) -> Result<()> {
        let content = self.root.join("content");

        info!("Loading content: {:?}", content);

        // load content
        let content = DirectoryLoader::new(&content, &content).load_from()?;

        // convert to value
        self.full_content = content.to_value()?;
        self.compact_content = Generator::compact_content(&self.full_content).unwrap_or_default();

        // dump content
        let writer = File::create(self.output().join("content.yaml"))?;
        serde_yaml::to_writer(writer, &self.full_content)?;
        let writer = File::create(self.output().join("compact.yaml"))?;
        serde_yaml::to_writer(writer, &self.compact_content)?;

        // done
        Ok(())
    }

    // Compact the content tree to contain only "content" sections.
    fn compact_content(v: &Value) -> Option<Value> {
        match v {
            Value::Object(m) => match m.get("content") {
                Some(Value::Object(mc)) => {
                    let mut result = Map::new();
                    for (k, v) in mc {
                        if let Some(x) = Generator::compact_content(v) {
                            result.insert(k.clone(), x);
                        }
                    }
                    Some(Value::Object(result))
                }
                Some(x) => Some(x.clone()),
                _ => None,
            },
            _ => Some(v.clone()),
        }
    }

    fn build(&self, render: &Render) -> Result<()> {
        // render all rules
        info!("Rendering content");
        for rule in &render.rules {
            self.render_rule(&rule)?;
        }

        // process assets
        info!("Processing assets");
        for a in &render.assets {
            self.process_asset(a)?;
        }

        info!("Done");
        // done
        Ok(())
    }

    fn process_asset(&self, asset: &Asset) -> Result<()> {
        let from = self.root.join(&asset.dir);

        let mut target = self.root.join("output");
        if let Some(ref to) = asset.to {
            target = target.join(to);
        }

        info!("Copying assets: {:?} -> {:?}", &from, &target);

        fs::create_dir_all(&target)?;

        copy::copy_dir(&from, &target, asset.glob.as_ref())?;

        Ok(())
    }

    fn render_rule(&self, rule: &Rule) -> Result<()> {
        info!(
            "Render rule: {:?}:{:?} -> {} -> {}",
            rule.selector_type, rule.selector, rule.template, rule.output_pattern
        );

        let result = rule.processor()?.query(&self.full_content)?;

        info!("Matches {} entries", result.len());

        // process selected entries
        for entry in result {
            info!("Processing entry: {}", entry);
            self.process_render(rule, entry)?;
        }

        // done
        Ok(())
    }

    fn process_render(&self, rule: &Rule, context: &Value) -> Result<()> {
        // eval
        let path = self
            .handlebars
            .render_template(&rule.output_pattern, context)?;
        let path = normalize_path(path);
        let template = self.handlebars.render_template(&rule.template, context)?;
        let target = RelativePath::new(&path).to_path(self.output());

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        // page data

        let output = Output::new(&self.site_url, &path);
        let output = serde_json::to_value(&output)?;

        // render
        info!("Render '{}' with '{}'", path, template);

        info!("  Target: {:?}", target);
        let writer = File::create(target)?;

        self.handlebars
            .render_to_write(&template, &self.data(&output, context), writer)?;

        // done
        Ok(())
    }

    fn data(&self, output: &Value, context: &Value) -> Value {
        let mut data = serde_json::value::Map::new();

        data.insert("output".into(), output.clone());
        data.insert("context".into(), context.clone());
        data.insert("full".into(), self.full_content.clone());
        data.insert("compact".into(), self.compact_content.clone());

        serde_json::value::Value::Object(data)
    }

    pub fn clean(&self) -> Result<()> {
        let p = self.output();
        let p = p.as_path();

        if p.exists() {
            info!("Cleaning up: {:?}", self.output());
            fs::remove_dir_all(self.output().as_path())?;
        }

        fs::create_dir_all(p)?;

        Ok(())
    }
}

fn normalize_path<S: AsRef<str>>(path: S) -> String {
    let s = path.as_ref().replace('\\', "/");
    RE.replace_all(&s, "/").into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        assert_eq!(normalize_path(""), "");
    }

    #[test]
    fn test_backslash() {
        assert_eq!(normalize_path("\\foo/bar/baz"), "/foo/bar/baz");
    }

    #[test]
    fn test_double() {
        assert_eq!(normalize_path("//foo/bar/baz"), "/foo/bar/baz");
    }

    #[test]
    fn test_double_2() {
        assert_eq!(normalize_path("//foo////bar/baz"), "/foo/bar/baz");
    }

    #[test]
    fn test_double_back() {
        assert_eq!(normalize_path("\\\\foo/bar/baz"), "/foo/bar/baz");
    }

    #[test]
    fn test_double_back_2() {
        assert_eq!(normalize_path("\\\\foo//bar/baz"), "/foo/bar/baz");
    }
}
