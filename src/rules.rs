use failure::Error;
use std::fs::File;

use serde_yaml;
use std::io;

use jsonpath_lib::Selector;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::Path;

use crate::error::GeneratorError;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub selector_type: String,
    pub selector: Option<String>,
    pub template: String,
    pub output_pattern: String,
}

pub trait RuleProcessor {
    fn query<'a>(&self, content: &'a Value) -> Result<Vec<&'a Value>>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Render {
    pub site: Site,
    pub rules: Vec<Rule>,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Site {
    pub basename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub dir: String,
    pub to: Option<String>,
    pub glob: Option<String>,
}

impl Render {
    pub fn load<R: io::Read>(reader: R) -> Result<Render> {
        let result = serde_yaml::from_reader(reader)?;
        Ok(result)
    }
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Render> {
        let reader = File::open(path)?;
        Self::load(reader)
    }
}

impl Rule {
    pub fn processor(&self) -> Result<Box<dyn RuleProcessor>> {
        match self.selector_type.as_str() {
            "layout" => Ok(Box::new(LayoutProcessor {
                layout: self.selector.clone(),
            })),
            "jsonpath" => Ok(Box::new(JsonPathProcessor {
                path: self
                    .selector
                    .clone()
                    .ok_or(GeneratorError::Error("Missing 'selector' value".into()))?,
            })),
            _ => Err(GeneratorError::Error(format!(
                "Unknown selector type: {}",
                self.selector_type
            ))
            .into()),
        }
    }
}

pub struct JsonPathProcessor {
    path: String,
}

impl RuleProcessor for JsonPathProcessor {
    fn query<'a>(&self, content: &'a Value) -> Result<Vec<&'a Value>> {
        let mut selector = Selector::new();
        let selector = selector
            .str_path(&self.path)
            .map_err(|e| GeneratorError::from(e))?;

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
}

pub struct LayoutProcessor {
    layout: Option<String>,
}

impl LayoutProcessor {
    fn is_layout(&self, item: &Map<String, Value>) -> bool {
        if let Some(layout) = item
            .get("frontMatter")
            .and_then(|v| v.as_object())
            .and_then(|fm| fm.get("layout"))
            .and_then(|v| v.as_str())
        {
            if let Some(required_layout) = &self.layout {
                required_layout.eq(layout)
            } else {
                true
            }
        } else {
            false
        }
    }

    fn find<'a>(&self, current: &'a Value, result: &mut Vec<&'a Value>) {
        if let Some(o) = current.as_object() {
            if self.is_layout(o) {
                result.push(current);
            }
            for (_, v) in o {
                self.find(v, result);
            }
        }
    }
}

impl RuleProcessor for LayoutProcessor {
    fn query<'a>(&self, content: &'a Value) -> Result<Vec<&'a Value>> {
        let mut result = Vec::new();

        self.find(content, &mut result);

        Ok(result)
    }
}
