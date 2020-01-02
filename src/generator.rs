use std::path::{Path, PathBuf};
use std::{fs, io};

use handlebars::Handlebars;

use log::{debug, info};

use crate::loader::directory::DirectoryLoader;
use crate::loader::Loader;
use failure::Error;
use std::fs::File;

type Result<T> = std::result::Result<T, Error>;

pub struct Generator<'a> {
    root: PathBuf,
    handlebars: Handlebars<'a>,
}

impl Generator<'_> {
    fn output(&self) -> PathBuf {
        self.root.join("output")
    }

    pub fn new(root: &Path) -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        Generator {
            root: root.to_path_buf(),
            handlebars,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.handlebars
            .register_templates_directory(".hbs", self.root.join("templates"))?;

        // clean output
        self.clean()?;

        // load data
        self.load_content()?;

        // render pages

        Ok(())
    }

    fn load_content(&mut self) -> Result<()> {
        let content = self.root.join("content");

        info!("Loading content: {:?}", content);

        let content = DirectoryLoader::new(content).load_from()?;
        serde_yaml::to_writer(io::stdout(), &content)?;

        // dump content
        let writer = File::create(self.output().join("content.yaml"))?;
        serde_yaml::to_writer(writer, &content)?;

        // done

        Ok(())
    }

    pub fn clean(&self) -> Result<()> {
        let p = self.output();
        let p = p.as_path();

        if p.exists() {
            fs::remove_dir_all(self.output().as_path())?;
        }

        fs::create_dir_all(p)?;

        Ok(())
    }
}
