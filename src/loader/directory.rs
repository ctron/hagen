use log::{debug, info};
use std::fs;

use std::path::Path;

use failure::Error;
type Result<T> = std::result::Result<T, Error>;

use crate::loader::{detect, Content, Loader, Metadata};
use serde_json::Value;
use std::collections::BTreeMap;

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
                let child_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default();
                content.insert(child_name, child);
            }
        }

        Ok(Content {
            metadata: Metadata {
                name: path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                path: path
                    .parent()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                filename: path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            },
            frontMatter: BTreeMap::new(),
            content: serde_json::to_value(content)?,
        })
    }
}
