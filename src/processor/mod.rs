use crate::generator::GeneratorConfig;
use failure::Error;
use handlebars::Handlebars;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use relative_path::RelativePath;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::io::Write;

pub mod rss;
pub mod sitemap;

type Result<T> = std::result::Result<T, Error>;

pub trait Processor {
    fn create<'a, 'reg>(
        &self,
        handlebars: &'reg mut Handlebars,
        data: &Value,
        config: &'a GeneratorConfig,
        processor_config: Value,
    ) -> Result<Box<dyn ProcessorContext + 'a>>;
}

pub trait ProcessorContext {
    fn file_created(
        &mut self,
        path: &RelativePath,
        context: &Value,
        handlebars: &mut Handlebars,
    ) -> Result<()>;
    fn complete(&mut self, handlebars: &mut Handlebars) -> Result<()>;
}

pub struct ProcessorSession<'a> {
    processors: Vec<Box<dyn ProcessorContext + 'a>>,
}

impl<'a> ProcessorSession<'a> {
    pub fn new<'reg>(
        processors: &BTreeMap<String, Box<dyn Processor>>,
        handlebars: &'reg mut Handlebars,
        data: &Value,
        config: &'a GeneratorConfig,
        processor_configs: &Map<String, Value>,
    ) -> Result<ProcessorSession<'a>> {
        let processors: Result<Vec<Box<dyn ProcessorContext + 'a>>> = processors
            .into_iter()
            .map(|(k, p)| {
                processor_configs
                    .get(k)
                    .map(|c| p.create(handlebars, data, config, c.clone()))
            })
            .filter_map(|o| o)
            .collect();
        Ok(ProcessorSession {
            processors: processors?,
        })
    }

    pub fn file_created(
        &mut self,
        path: &RelativePath,
        context: &Value,
        handlebars: &mut Handlebars,
    ) -> Result<()> {
        for p in &mut self.processors {
            (*p).file_created(path, context, handlebars)?;
        }
        Ok(())
    }

    pub fn complete(&mut self, handlebars: &mut Handlebars) -> Result<()> {
        for p in &mut self.processors {
            (*p).complete(handlebars)?;
        }
        Ok(())
    }
}

pub fn xml_write_element<'a, S1, S2, W>(writer: &mut Writer<W>, name: S1, value: S2) -> Result<()>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
    W: Write,
{
    writer.write_event(Event::Start(BytesStart::borrowed_name(
        name.as_ref().as_bytes(),
    )))?;

    writer.write_event(Event::Text(BytesText::from_plain_str(value.as_ref())))?;

    writer.write_event(Event::End(BytesEnd::borrowed(name.as_ref().as_bytes())))?;

    writer.write(b"\n")?;

    Ok(())
}
