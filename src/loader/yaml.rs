use failure::Error;
use log::{debug, info};
use std::path::Path;

type Result<T> = std::result::Result<T, Error>;

use crate::loader::{Content, Loader, Metadata};
use serde::Deserializer;
use std::collections::BTreeMap;
use std::fs::File;

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
        let path = self.path.as_ref();
        info!("Loading - YAML: {:?}", path);

        let reader = File::open(path)?;
        let content = serde_yaml::from_reader(reader)?;

        Ok(Content {
            metadata: Metadata::from_path(path, "yaml"),
            front_matter: BTreeMap::new(),
            content,
        })
    }
}
