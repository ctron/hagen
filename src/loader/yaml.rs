use failure::Error;
use log::{debug, info};
use std::path::Path;

type Result<T> = std::result::Result<T, Error>;

use crate::loader::{Content, JsonBodyProvider, Loader, Metadata};

use serde_json::{Map, Value};
use std::fmt::Debug;
use std::fs::File;

pub struct YAMLLoader<P1: AsRef<Path>, P2: AsRef<Path>> {
    root: P1,
    path: P2,
}

impl<P1: AsRef<Path>, P2: AsRef<Path>> YAMLLoader<P1, P2> {
    pub fn new(root: P1, path: P2) -> Self {
        YAMLLoader { root, path }
    }
}

impl<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug> Loader for YAMLLoader<P1, P2> {
    fn load_from(&self) -> Result<Content> {
        let path = self.path.as_ref();
        info!("Loading - YAML: {:?}", path);

        let reader = File::open(path)?;
        let content: Value = serde_yaml::from_reader(reader)?;

        Ok(Content {
            metadata: Metadata::from_path(&self.root, path, path.file_stem(), "yaml"),
            front_matter: Map::new(),
            content: Box::new(JsonBodyProvider::new(content)),
        })
    }
}
