use crate::generator::GeneratorContext;
use failure::Error;
use relative_path::RelativePath;
use std::path::Path;

pub mod sitemap;

type Result<T> = std::result::Result<T, Error>;

pub trait Processor {
    fn create<'a>(&self, context: &'a GeneratorContext) -> Result<Box<dyn ProcessorContext + 'a>>;
}

pub trait ProcessorContext {
    fn file_created(&mut self, path: &RelativePath) -> Result<()>;
    fn complete(&mut self) -> Result<()>;
}

pub struct ProcessorSession<'a> {
    processors: Vec<Box<dyn ProcessorContext + 'a>>,
}

impl<'a> ProcessorSession<'a> {
    pub fn new(
        processors: &Vec<Box<dyn Processor>>,
        context: &'a GeneratorContext,
    ) -> Result<ProcessorSession<'a>> {
        let processors: Result<Vec<Box<dyn ProcessorContext + 'a>>> =
            processors.into_iter().map(|p| p.create(context)).collect();
        Ok(ProcessorSession {
            processors: processors?,
        })
    }

    pub fn file_created(&mut self, path: &RelativePath) -> Result<()> {
        for p in &mut self.processors {
            (*p).file_created(path)?;
        }
        Ok(())
    }

    pub fn complete(&mut self) -> Result<()> {
        for p in &mut self.processors {
            (*p).complete()?;
        }
        Ok(())
    }
}