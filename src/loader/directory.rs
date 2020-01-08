use log::{debug, info};
use std::fs;

use std::path::Path;

use failure::Error;
type Result<T> = std::result::Result<T, Error>;

use crate::loader::{
    detect, path_to_string, BodyProvider, Content, JsonBodyProvider, Loader, Metadata,
};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub struct DirectoryBodyProvider {
    body: BTreeMap<String, Content>,
}

impl BodyProvider for DirectoryBodyProvider {
    fn body(&self) -> Result<Value> {
        let mut m = Map::with_capacity(self.body.len());

        for (k, v) in &self.body {
            m.insert(k.clone(), v.to_value()?);
        }

        Ok(Value::Object(m))
    }
}

pub struct DirectoryLoader<P: AsRef<Path>> {
    path: P,
}

impl<P: AsRef<Path>> DirectoryLoader<P> {
    pub fn new(path: P) -> Self {
        DirectoryLoader { path }
    }
}

impl<P: AsRef<Path>> Loader for DirectoryLoader<P> {
    fn load_from(&self) -> Result<Content> {
        let path = self.path.as_ref();
        info!("Loading - directory: {:?}", path);

        let mut content: BTreeMap<String, Content> = BTreeMap::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;

            let path = entry.path();
            if let Some(loader) = detect(path.clone()) {
                let child = loader.load_from()?;
                let child_name = child.metadata.name.clone();
                content.insert(child_name, child);
            }
        }

        Ok(Content {
            metadata: Metadata::from_path(path, path.file_name(), "directory"),
            front_matter: Map::new(),
            content: Box::new(DirectoryBodyProvider { body: content }),
        })
    }
}
