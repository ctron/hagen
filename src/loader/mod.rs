use std::path::Path;

use crate::loader::directory::DirectoryLoader;
use crate::loader::yaml::YAMLLoader;

use serde::{Deserialize, Serialize};

use failure::Error;
use std::collections::BTreeMap;

type Result<T> = std::result::Result<T, Error>;

pub mod directory;
pub mod yaml;

pub trait Loader {
    fn load_from(&self) -> Result<Content>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    name: String,
    path: String,
    filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    metadata: Metadata,
    frontMatter: BTreeMap<String, serde_json::Value>,
    content: serde_json::Value,
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
            _ => None,
        },
    }
}
