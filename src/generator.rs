use failure::Error;
use std::fs;
use std::path::{Path, PathBuf};

use handlebars::Handlebars;

type Result<T> = std::result::Result<T, Error>;

pub struct Generator {
    root: PathBuf,
}

impl Generator {
    fn output(&self) -> PathBuf {
        self.root.join("output")
    }

    pub fn new(root: &Path) -> Generator {
        Generator {
            root: root.to_path_buf(),
        }
    }

    pub fn run(&self) -> Result<()> {
        self.clean()?;

        Ok(())
    }

    pub fn clean(&self) -> Result<()> {
        let p = self.output();
        let p = p.as_path();

        if p.exists() {
            fs::remove_dir_all(self.output().as_path())?;
        }

        Ok(())
    }
}
