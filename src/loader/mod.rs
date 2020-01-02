use std::path::Path;

use crate::loader::directory::DirectoryLoader;
use crate::loader::yaml::YAMLLoader;

use serde::{Deserialize, Serialize};

use failure::Error;
use std::collections::BTreeMap;
use std::ffi::OsStr;

type Result<T> = std::result::Result<T, Error>;

pub mod directory;
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
    pub fn from_path<P: AsRef<Path>, S: Into<String>>(path: P, type_name: S) -> Metadata {
        let path = path.as_ref();
        Metadata {
            name: path_to_string(path.file_stem()),
            path: path_to_string(path.parent()),
            filename: path_to_string(path.file_name()),
            type_name: type_name.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    metadata: Metadata,
    front_matter: BTreeMap<String, serde_json::Value>,
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
