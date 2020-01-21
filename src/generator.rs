use std::path::{Path, PathBuf};
use std::{env, fs};

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

use crate::processor::{Processor, ProcessorSession};

use crate::helper::sort::SortedHelper;
use crate::processor::sitemap::SitemapProcessor;
use clap::Clap;
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;
use url::Url;

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

#[derive(Clone, Clap)]
#[clap(version = "0.1.0", author = "Jens Reimann")]
pub struct Options {
    /// Override the basename of the site
    #[clap(short = "b", long = "base")]
    basename: Option<String>,

    /// The root of the site. Must contain the file "render.yaml" and the "content" directory.
    #[clap(short = "r", long = "root")]
    root: Option<String>,

    /// Dump the content files as well.
    #[clap(short = "D", long = "dump")]
    dump: bool,
}

pub struct GeneratorContext<'a> {
    pub root: &'a Path,
    pub output: &'a Path,
    pub basename: &'a Url,
}

pub struct Generator<'a> {
    options: Options,
    root: PathBuf,

    handlebars: Handlebars<'a>,

    processors: Vec<Box<dyn Processor>>,

    config: Option<Render>,
    full_content: Value,
    compact_content: Value,
}

impl Generator<'_> {
    fn output(&self) -> PathBuf {
        self.root.join("output")
    }

    pub fn new(options: Options) -> Self {
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

        // register processors

        let mut processors: Vec<Box<dyn Processor>> = Vec::new();
        processors.push(Box::new(SitemapProcessor::new()));

        // eval root

        let root = match options.root {
            Some(ref x) => PathBuf::from(x),
            None => env::current_dir().expect("Failed to get current directory"),
        };

        // create generator

        Generator {
            options,
            handlebars,
            root,
            processors,

            config: Default::default(),
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

        // load config
        self.load_config()?;

        // build
        self.build()?;

        // done
        Ok(())
    }

    fn load_config(&mut self) -> Result<()> {
        info!("Loading render rules");
        self.config = Some(Render::load_from(self.root.join("render.yaml"))?);

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

        if self.options.dump {
            // dump content
            info!("Dumping content");
            let writer = File::create(self.output().join("content.yaml"))?;
            serde_yaml::to_writer(writer, &self.full_content)?;
            let writer = File::create(self.output().join("compact.yaml"))?;
            serde_yaml::to_writer(writer, &self.compact_content)?;
        }

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

    fn build(&self) -> Result<()> {
        let config = self
            .config
            .as_ref()
            .ok_or(GeneratorError::Error("Missing site configuration".into()))?;

        let mut basename = (&self.options.basename)
            .as_ref()
            .unwrap_or(&config.site.basename)
            .to_owned();

        if !basename.ends_with('/') {
            basename.push('/');
        }

        let basename = Url::from_str(&basename)?;

        // context
        let context = GeneratorContext {
            basename: &basename,
            root: &self.root,
            output: &self.output(),
        };

        let mut processors = ProcessorSession::new(&self.processors, &context)?;

        // render all rules
        info!("Rendering content");
        for rule in &config.rules {
            self.render_rule(&rule, &mut processors, &context)?;
        }

        // process assets
        info!("Processing assets");
        for a in &config.assets {
            self.process_asset(a)?;
        }

        processors.complete()?;

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

    fn render_rule(
        &self,
        rule: &Rule,
        processors: &mut ProcessorSession,
        context: &GeneratorContext,
    ) -> Result<()> {
        info!(
            "Render rule: {:?}:{:?} -> {} -> {}",
            rule.selector_type, rule.selector, rule.template, rule.output_pattern
        );

        let result = rule.processor()?.query(&self.full_content)?;

        info!("Matches {} entries", result.len());

        // process selected entries
        for entry in result {
            info!("Processing entry: {}", entry);
            self.process_render(rule, entry, processors, context)?;
        }

        // done
        Ok(())
    }

    fn process_render(
        &self,
        rule: &Rule,
        context: &Value,
        processors: &mut ProcessorSession,
        generator_context: &GeneratorContext,
    ) -> Result<()> {
        // eval
        let path = self
            .handlebars
            .render_template(&rule.output_pattern, context)?;
        let path = normalize_path(path);
        let template = self.handlebars.render_template(&rule.template, context)?;

        let relative_target = RelativePath::new(&path);
        let target = relative_target.to_path(self.output());

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        // page data

        let output = Output::new(generator_context.basename.as_str(), &path);
        let output = serde_json::to_value(&output)?;

        // render
        info!("Render '{}' with '{}'", path, template);

        info!("  Target: {:?}", target);
        let writer = File::create(target)?;

        let context = Generator::build_context(&rule, &context)?;

        self.handlebars
            .render_to_write(&template, &self.data(output, context), writer)?;

        // call processors
        processors.file_created(&relative_target)?;

        // done
        Ok(())
    }

    /// Build the render content context object from the rules context mappings
    fn build_context(rule: &Rule, context: &Value) -> Result<Value> {
        if rule.context.is_empty() {
            return Ok(context.clone());
        }

        let mut result = Map::new();

        for (k, v) in &rule.context {
            match v {
                Value::String(path) => {
                    let obj = jsonpath_lib::select(context, &path)?;
                    let obj = obj.as_slice();
                    let value = match obj {
                        [] => None,
                        [x] => Some((*x).clone()),
                        obj => Some(Value::Array(obj.iter().cloned().cloned().collect())),
                    };
                    info!("Mapped context - name: {:?} = {:?}", k, value);
                    if let Some(value) = value {
                        result.insert(k.into(), value);
                    }
                }
                _ => {
                    return Err(GeneratorError::Error(
                        "Context value must be a string/JSON path".into(),
                    ))
                }
            }
        }

        Ok(Value::Object(result))
    }

    fn data(&self, output: Value, context: Value) -> Value {
        let mut data = serde_json::value::Map::new();

        // add the output context
        data.insert("output".into(), output);
        data.insert("context".into(), context);
        // add the full content tree
        data.insert("full".into(), self.full_content.clone());
        // add the compact content tree
        data.insert("compact".into(), self.compact_content.clone());

        // convert to json object
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
