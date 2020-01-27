use std::path::PathBuf;
use std::{env, fs};

use handlebars::Handlebars;

use log::{debug, info};

use crate::error::GeneratorError;
use crate::loader::directory::DirectoryLoader;
use crate::loader::Loader;
use crate::rules::{Asset, Render, Rule};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use std::fs::File;

type Result<T> = std::result::Result<T, GeneratorError>;

use crate::helper::basic::{ConcatHelper, DumpHelper, ExpandHelper, TimesHelper};
use crate::helper::markdown::MarkdownifyHelper;

use crate::copy;
use crate::helper::time::TimeHelper;
use crate::helper::url::{AbsoluteUrlHelper, ActiveHelper, RelativeUrlHelper};
use relative_path::RelativePath;

use crate::processor::{Processor, ProcessorSession};

use crate::helper::sort::SortedHelper;
use crate::processor::rss::RssProcessor;
use crate::processor::sitemap::SitemapProcessor;
use clap::Clap;
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use url::Url;

lazy_static! {
    static ref RE: Regex = Regex::new(r"/{2,}").unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Clone, Clap)]
#[clap(version = "0.1.0", author = "Jens Reimann")]
pub struct Options {
    /// Override the basename of the site
    #[clap(short = "b", long = "base")]
    basename: Option<String>,

    /// The root of the site. Must contain the file "hagen.yaml" and the "content" directory.
    #[clap(short = "r", long = "root")]
    root: Option<String>,

    /// Dump the content files as well.
    #[clap(short = "D", long = "dump")]
    dump: bool,
}

#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub root: PathBuf,
    pub output: PathBuf,
    pub basename: Url,
}

#[derive(Debug, Clone)]
pub struct GeneratorContext {
    pub config: GeneratorConfig,
    pub output: Output,
}

impl GeneratorContext {
    pub fn new(config: &GeneratorConfig, output: &Output) -> Self {
        GeneratorContext {
            config: config.clone(),
            output: output.clone(),
        }
    }
}

pub struct Generator<'a> {
    options: Options,
    root: PathBuf,

    handlebars: Handlebars<'a>,

    processors: Vec<Box<dyn Processor>>,

    config: Option<Render>,
    full_content: Value,
    compact_content: Value,

    context_provider: Arc<RwLock<Option<GeneratorContext>>>,
}

impl<'a> Generator<'a> {
    fn output(&self) -> PathBuf {
        self.root.join("output")
    }

    pub fn new(options: Options) -> Self {
        // create instance

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        // register processors

        let mut processors: Vec<Box<dyn Processor>> = Vec::new();
        processors.push(Box::new(SitemapProcessor::new(
            "$.context.page.frontMatter.timestamp.published",
            "$.context.page.frontMatter.timestamp.updated",
        )));
        processors.push(Box::new(RssProcessor::new()));

        // eval root

        let root = match options.root {
            Some(ref x) => PathBuf::from(x),
            None => env::current_dir().expect("Failed to get current directory"),
        };

        // context

        let context_provider = Arc::new(RwLock::new(None));

        // register helpers

        handlebars.register_helper("dump", Box::new(DumpHelper));

        handlebars.register_helper("times", Box::new(TimesHelper));
        handlebars.register_helper("expand", Box::new(ExpandHelper));
        handlebars.register_helper("concat", Box::new(ConcatHelper));

        handlebars.register_helper("sorted", Box::new(SortedHelper));

        handlebars.register_helper("markdownify", Box::new(MarkdownifyHelper));

        handlebars.register_helper("timestamp", Box::new(TimeHelper));

        handlebars.register_helper(
            "absolute_url",
            Box::new(AbsoluteUrlHelper {
                context: context_provider.clone(),
            }),
        );
        handlebars.register_helper(
            "relative_url",
            Box::new(RelativeUrlHelper {
                context: context_provider.clone(),
            }),
        );
        handlebars.register_helper(
            "active",
            Box::new(ActiveHelper {
                context: context_provider.clone(),
            }),
        );

        // create generator

        Generator {
            options,
            handlebars,
            root,
            processors,

            config: Default::default(),
            full_content: Default::default(),
            compact_content: Default::default(),
            context_provider: context_provider.clone(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        debug!("Running generator");

        let templates = self.root.join("templates");
        info!("Loading templates: {:?}", templates);
        self.handlebars
            .register_templates_directory(".hbs", templates)?;

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
        let path = self.root.join("hagen.yaml");
        info!("Loading configuration: {:?}", path);
        self.config = Some(Render::load_from(path)?);

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

    fn build(&mut self) -> Result<()> {
        let config = self
            .config
            .as_ref()
            .ok_or(GeneratorError::Error("Missing site configuration".into()))?
            .clone();

        let mut basename = (&self.options.basename)
            .as_ref()
            .unwrap_or(&config.site.basename)
            .to_owned();

        if !basename.ends_with('/') {
            basename.push('/');
        }

        let basename = Url::from_str(&basename)?;

        // context
        let generator_config = GeneratorConfig {
            basename: basename.clone(),
            root: self.root.clone(),
            output: self.output(),
        };

        let mut processors = ProcessorSession::new(&self.processors, &generator_config)?;

        // render all rules
        info!("Rendering content");
        for rule in &config.rules {
            self.render_rule(&rule, &mut processors, &generator_config)?;
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
        &mut self,
        rule: &Rule,
        processors: &mut ProcessorSession,
        config: &GeneratorConfig,
    ) -> Result<()> {
        info!(
            "Render rule: {:?}:{:?} -> {:?} -> {}",
            rule.selector_type, rule.selector, rule.template, rule.output_pattern
        );

        let result: Vec<_> = rule.processor()?.query(&self.full_content)?;
        let result: Vec<Value> = result.iter().cloned().cloned().collect();

        info!("Matches {} entries", result.len());

        // process selected entries
        for entry in &result {
            debug!("Processing entry: {}", entry);
            self.process_render(rule, entry, processors, config)?;
        }

        // done
        Ok(())
    }

    fn process_render(
        &mut self,
        rule: &Rule,
        context: &Value,
        processors: &mut ProcessorSession,
        config: &GeneratorConfig,
    ) -> Result<()> {
        // eval
        let path = self
            .handlebars
            .render_template(&rule.output_pattern, context)?;
        let path = normalize_path(path);
        let template = rule
            .template
            .as_ref()
            .map(|t| self.handlebars.render_template(&t, context))
            .transpose()?;

        let relative_target = RelativePath::new(&path);
        let target = relative_target.to_path(self.output());

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        // page data

        let output = Output::new(config.basename.as_str(), &path);

        {
            let ctx = GeneratorContext::new(config, &output);
            {
                *self.context_provider.write().unwrap() = Some(ctx);
                // *Arc::make_mut(&mut self.context_provider) = Some(ctx);
            }
            let output = serde_json::to_value(&output)?;

            // render

            info!("Render '{}' with '{:?}'", path, template);
            info!("  Target: {:?}", target);

            let writer = File::create(target)?;

            let context = Generator::build_context(&rule, &context)?;
            let data = &self.data(output, context.clone());

            match template {
                Some(ref t) => self.handlebars.render_to_write(t, data, writer)?,
                None => {
                    let content = match &context.as_object().and_then(|s| s.get("content")) {
                    Some(Value::String(c)) => Ok(c),
                    _ => Err(GeneratorError::Error("Rule is missing 'template' on rule and '.content' value in context. Either must be set.".into())),
                }?;
                    self.handlebars
                        .render_template_to_write(content, data, writer)?;
                }
            }

            // call processors
            processors.file_created(&relative_target, data)?;

            // reset current context
            {
                *self.context_provider.write().unwrap() = None;
                // *Arc::make_mut(&mut self.context_provider) = None;
            }
        }

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
                    debug!("Mapped context - name: {:?} = {:?}", k, value);
                    if let Some(value) = value {
                        result.insert(k.into(), value);
                    }
                }
                _ => {
                    return Err(GeneratorError::Error(
                        "Context value must be a string/JSON path".into(),
                    ));
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
