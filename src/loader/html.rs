use failure::Error;
use log::info;
use std::path::Path;

use super::front_matter::parse_front_matter;

type Result<T> = std::result::Result<T, Error>;

use crate::loader::{Content, JsonBodyProvider, Loader, Metadata};

use std::fmt::Debug;
use std::fs::read_to_string;

pub struct HtmlLoader<P1: AsRef<Path>, P2: AsRef<Path>> {
    root: P1,
    path: P2,
}

impl<P1: AsRef<Path>, P2: AsRef<Path>> HtmlLoader<P1, P2> {
    pub fn new(root: P1, path: P2) -> Self {
        HtmlLoader { root, path }
    }
}

impl<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug> Loader for HtmlLoader<P1, P2> {
    fn load_from(&self) -> Result<Content> {
        let path = self.path.as_ref();
        info!("Loading - HTML: {:?}", path);

        let data = read_to_string(path)?;

        let front_matter = parse_front_matter(&data)?;

        Ok(Content {
            metadata: Metadata::from_path(&self.root, path, path.file_stem(), "html"),
            front_matter: front_matter.1.unwrap_or_default(),
            content: Box::new(JsonBodyProvider::new(serde_json::Value::String(
                front_matter.0,
            ))),
        })
    }
}
