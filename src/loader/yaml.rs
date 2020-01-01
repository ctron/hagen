use failure::Error;
use log::{debug, info};
use std::path::Path;

type Result<T> = std::result::Result<T, Error>;

use crate::loader::{Content, Loader, Metadata};
use serde_json::Value;
use std::collections::BTreeMap;

pub struct YAMLLoader<P: AsRef<Path>> {
    path: P,
}

impl<P: AsRef<Path>> YAMLLoader<P> {
    pub fn new(path: P) -> Self {
        YAMLLoader { path }
    }
}

impl<P: AsRef<Path>> Loader for YAMLLoader<P> {
    fn load_from(&self) -> Result<Content> {
        info!("Loading - YAML: {:?}", self.path.as_ref());

        Ok(Content {
            metadata: Metadata {
                path: String::from(""),
                name: String::from(""),
                filename: String::from(""),
            },
            frontMatter: BTreeMap::new(),
            content: Value::Null,
        })
    }
}
