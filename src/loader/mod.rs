use std::path::{Path, PathBuf};

use crate::loader::directory::DirectoryLoader;
use crate::loader::markdown::MarkdownLoader;
use crate::loader::yaml::YAMLLoader;

use serde::{Deserialize, Serialize};

use failure::Error;
use relative_path::RelativePath;
use serde_json::{Map, Value};
use std::ffi::OsStr;
use std::fmt::Debug;
use std::net::Shutdown::Read;

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
    parent: String,
    filename: String,

    #[serde(rename = "type")]
    type_name: String,
}

impl Metadata {
    pub fn from_path<P1, P2, S>(root: P1, path: P2, name: Option<&OsStr>, type_name: S) -> Metadata
    where
        P1: AsRef<Path> + Debug,
        P2: AsRef<Path> + Debug,
        S: Into<String>,
    {
        let path = path.as_ref();
        let parent = path_to_string(path.parent());
        let root = path_to_string(Some(root.as_ref()));

        let parent = if parent.starts_with(&root) {
            let len = root.len();
            if len == parent.len() {
                String::from("/")
            } else {
                let mut r = parent.clone();
                let len = root.len();
                r.replace_range(0..len, "");
                r
            }
        } else {
            parent
        };

        Metadata {
            parent,
            name: path_to_string(name),
            filename: path_to_string(path.file_name()),
            type_name: type_name.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_2() {
        let m = Metadata::from_path(&"/root", &"/root/foo/bar", None, "type");
        assert_eq!(m.parent, "/foo");
    }

    #[test]
    fn test_path_3() {
        let m = Metadata::from_path(&"/root", &"/root/foo/bar/baz.md", None, "type");
        assert_eq!(m.parent, "/foo/bar");
    }

    #[test]
    fn test_path_root() {
        let m = Metadata::from_path(&"/root", &"/root", None, "type");
        assert_eq!(m.parent, "/");
    }

    #[test]
    fn test_path_root_first() {
        let m = Metadata::from_path(&"/root", &"/root/foo", None, "type");
        assert_eq!(m.parent, "/");
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

pub trait BodyProvider {
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

pub fn detect<P1, P2>(root: P1, path: P2) -> Option<Box<dyn Loader>>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let path = path.as_ref().to_path_buf();

    if path.is_dir() {
        return Some(Box::new(DirectoryLoader::new(
            root.as_ref().to_path_buf().clone(),
            path.clone(),
        )));
    }

    match path.extension() {
        None => None,
        Some(str) => match str.to_str() {
            Some("yaml") | Some("yml") => {
                Some(Box::new(YAMLLoader::new(root.as_ref().to_path_buf(), path)))
            }
            Some("md") => Some(Box::new(MarkdownLoader::new(
                root.as_ref().to_path_buf(),
                path,
            ))),
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
