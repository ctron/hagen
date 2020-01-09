use std::fs;
use std::path::{Path, PathBuf};

use handlebars::Handlebars;

use log::{debug, info};

use crate::loader::directory::DirectoryLoader;
use crate::loader::{Content, Loader};
use crate::rules::{Asset, Render, Rule};

use crate::error;
use crate::error::GeneratorError;
use crate::error::GeneratorError::GenericError;
use failure::Error;
use jsonpath_lib::Selector;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs::File;

use fs_extra::dir::{copy, CopyOptions};

type Result<T> = std::result::Result<T, Error>;

use crate::helper::basic::{ExpandHelper, RelativeUrlHelper, TimesHelper};
use crate::helper::markdown::MarkdownifyHelper;
use fs_extra::copy_items;

use crate::copy;

pub struct Generator<'a> {
    root: PathBuf,
    handlebars: Handlebars<'a>,
    content: Option<Content>,

    full_content: Value,
    compact_content: Value,
}

impl Generator<'_> {
    fn output(&self) -> PathBuf {
        self.root.join("output")
    }

    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        // create instance

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        // register helpers

        handlebars.register_helper("times", Box::new(TimesHelper));
        handlebars.register_helper("expand", Box::new(ExpandHelper));
        handlebars.register_helper("relative_url", Box::new(RelativeUrlHelper));

        handlebars.register_helper("markdownify", Box::new(MarkdownifyHelper));

        // create generator

        Generator {
            root: root.as_ref().to_path_buf(),
            handlebars,
            content: Default::default(),
            full_content: Default::default(),
            compact_content: Default::default(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
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
        let content = DirectoryLoader::new(content).load_from()?;

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
            Value::Object(m) => match m.get("content".into()) {
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
        let target = self.root.join("output").join(&asset.to);

        info!("Copying assets: {:?} -> {:?}", &from, &target);

        fs::create_dir_all(&target)?;

        let mut options = CopyOptions::new();
        options.copy_inside = true;
        copy::copy(from, target, &options)?;

        Ok(())
    }

    fn render_rule(&self, rule: &Rule) -> Result<()> {
        info!(
            "Render rule: {} -> {} -> {}",
            rule.selector, rule.template, rule.output_pattern
        );

        let result = query(&rule.selector, &self.full_content)?;

        info!("Matches {} entries", result.len());

        // process selected entries
        for entry in result {
            info!("Processing entry: {}", entry);
            self.process_render(rule, entry)?;
        }

        // done
        Ok(())
    }

    fn process_render(&self, rule: &Rule, entry: &Value) -> Result<()> {
        // eval
        let path = self
            .handlebars
            .render_template(&rule.output_pattern, entry)?;
        let template = self.handlebars.render_template(&rule.template, entry)?;
        let target = self.output().join(Path::new(&path));

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        // render
        info!("Render '{}' with '{}'", path, template);

        info!("  Target: {:?}", target);
        let writer = File::create(target)?;

        self.handlebars
            .render_to_write(&template, &self.data(entry), writer)?;

        // done
        Ok(())
    }

    fn data(&self, entry: &Value) -> Value {
        let mut data = serde_json::value::Map::new();
        data.insert("page".into(), entry.clone());
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

fn query<'a>(s: &'a str, content: &'a Value) -> Result<Vec<&'a Value>> {
    let mut selector = Selector::new();
    let selector = selector.str_path(&s).map_err(|e| GeneratorError::from(e))?;

    match selector.value(&content).select() {
        Err(err) => Err(GeneratorError::from(err).into()),
        Ok(v) => Ok(v),
    }
    .map(|v| -> Vec<&Value> {
        let mut v = v.clone();
        v.retain(|e| e.is_object());
        v
    })
}

/*
fn query_x<'a>(s: &'a str, content: &'a Value) -> Result<Vec<&'a Value>> {
    let result = jq_rs::run(str, serde_json::to_string(content)?.into())?;
    serde_json::from_str(&result)?
}
*/
