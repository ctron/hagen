use std::path::Path;

use crate::loader::directory::DirectoryLoader;
use crate::loader::markdown::MarkdownLoader;
use crate::loader::yaml::YAMLLoader;

use serde::{Deserialize, Serialize};

use failure::Error;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::ffi::OsStr;

type Result<T> = std::result::Result<T, Error>;

pub mod directory;
pub mod markdown;
pub mod yaml;

pub trait Loader {
    fn load_from(&self) -> Result<Content>;
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Metadata {
    name: String,
    path: String,
    filename: String,

    #[serde(rename = "type")]
    type_name: String,
}

impl Metadata {
    pub fn from_path<P: AsRef<Path>, S: Into<String>>(
        path: P,
        name: Option<&OsStr>,
        type_name: S,
    ) -> Metadata {
        let path = path.as_ref();
        Metadata {
            path: path_to_string(path.parent()),
            name: path_to_string(name),
            filename: path_to_string(path.file_name()),
            type_name: type_name.into(),
        }
    }
}

pub struct Content {
    pub metadata: Metadata,
    pub front_matter: serde_json::Map<String, Value>,
    pub content: Box<dyn BodyProvider>,
}

impl Content {
    pub fn to_value(&self) -> Result<Value> {
        let mut m = Map::new();

        m.insert("metadata".into(), serde_json::to_value(&self.metadata)?);
        m.insert(
            "frontMatter".into(),
            Value::Object(self.front_matter.clone()),
        );
        m.insert("content".into(), self.content.body()?);

        Ok(Value::Object(m))
    }
}

trait BodyProvider {
    fn body(&self) -> Result<Value>;
}

pub struct JsonBodyProvider {
    body: Value,
}

impl JsonBodyProvider {
    fn new(body: Value) -> JsonBodyProvider {
        JsonBodyProvider { body }
    }
}

impl BodyProvider for JsonBodyProvider {
    fn body(&self) -> Result<Value> {
        Ok(self.body.clone())
    }
}

pub fn detect<P>(path: P) -> Option<Box<dyn Loader>>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().to_path_buf();

    if path.is_dir() {
        return Some(Box::new(DirectoryLoader::new(path)));
    }

    match path.extension() {
        None => None,
        Some(str) => match str.to_str() {
            Some("yaml") | Some("yml") => Some(Box::new(YAMLLoader::new(path))),
            Some("md") => Some(Box::new(MarkdownLoader::new(path))),
            _ => None,
        },
    }
}

fn path_to_string<P: AsRef<OsStr>>(path: Option<P>) -> String {
    match path {
        None => String::default(),
        Some(s) => s
            .as_ref()
            .to_str()
            .map(|s| s.to_string())
            .unwrap_or_default(),
    }
}
