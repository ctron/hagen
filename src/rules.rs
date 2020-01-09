use failure::Error;
use std::fs::File;

use serde_yaml;
use std::io;

use serde::{Deserialize, Serialize};
use std::path::Path;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub selector: String,
    pub template: String,
    pub output_pattern: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Render {
    pub rules: Vec<Rule>,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub dir: String,
    pub to: String,
}

impl Render {
    pub fn load<R: io::Read>(reader: R) -> Result<Render> {
        let result = serde_yaml::from_reader(reader)?;
        Ok(result)
    }
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Render> {
        let reader = File::open(path)?;
        Self::load(reader)
    }
}
