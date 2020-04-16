use failure::Error;
use log::info;
use std::path::Path;

use super::front_matter::parse_front_matter;

type Result<T> = std::result::Result<T, Error>;

use crate::loader::{Content, JsonBodyProvider, Loader, Metadata};

use std::fmt::Debug;
use std::fs::read_to_string;

pub struct PlainLoader<P1: AsRef<Path>, P2: AsRef<Path>> {
    root: P1,
    path: P2,
    type_name: String,
    with_front_matter: bool,
}

impl<P1: AsRef<Path>, P2: AsRef<Path>> PlainLoader<P1, P2> {
    pub fn new<S1: Into<String>>(
        root: P1,
        path: P2,
        type_name: S1,
        with_front_matter: bool,
    ) -> Self {
        PlainLoader {
            root,
            path,
            type_name: type_name.into(),
            with_front_matter,
        }
    }
}

impl<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug> Loader for PlainLoader<P1, P2> {
    fn load_from(&self) -> Result<Content> {
        let path = self.path.as_ref();
        info!("Loading - Plain: {:?}", path);

        let data = read_to_string(path)?;

        let front_matter = if self.with_front_matter {
            parse_front_matter(&data)?
        } else {
            (data, None)
        };

        Ok(Content {
            metadata: Metadata::from_path(&self.root, path, path.file_stem(), &self.type_name),
            front_matter: front_matter.1.unwrap_or_default(),
            content: Box::new(JsonBodyProvider::new(serde_json::Value::String(
                front_matter.0,
            ))),
        })
    }
}
