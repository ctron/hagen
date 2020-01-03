use failure::Error;
use log::{debug, info};
use std::path::Path;

type Result<T> = std::result::Result<T, Error>;

use crate::loader::{Content, Loader, Metadata};

use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fs::{read_to_string, File};

pub struct MarkdownLoader<P: AsRef<Path>> {
    path: P,
}

impl<P: AsRef<Path>> MarkdownLoader<P> {
    pub fn new(path: P) -> Self {
        MarkdownLoader { path }
    }
}

impl<P: AsRef<Path>> Loader for MarkdownLoader<P> {
    fn load_from(&self) -> Result<Content> {
        let path = self.path.as_ref();
        info!("Loading - Markdown: {:?}", path);

        let data = read_to_string(path)?;

        let front_matter = parse_front_matter(&data);

        Ok(Content {
            metadata: Metadata::from_path(path, "md"),
            front_matter: front_matter.1.unwrap_or_default(),
            content: serde_json::Value::String(front_matter.0),
        })
    }
}

fn parse_front_matter(data: &String) -> (String, Option<Map<String, Value>>) {
    todo!("Implement front matter parser");
    (data.clone(), None)
}
