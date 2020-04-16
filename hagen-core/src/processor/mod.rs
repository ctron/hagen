use crate::generator::{GeneratorConfig, Output};
use crate::path::first_value_for_path;
use failure::Error;
use handlebars::Handlebars;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
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
        output: &Output,
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
        output: &Output,
        context: &Value,
        handlebars: &mut Handlebars,
    ) -> Result<()> {
        for p in &mut self.processors {
            (*p).file_created(output, context, handlebars)?;
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
    xml_write_element_raw(
        writer,
        name,
        Event::Text(BytesText::from_plain_str(value.as_ref())),
    )
}

#[allow(dead_code)]
pub fn xml_write_element_cdata<'a, S1, S2, W>(
    writer: &mut Writer<W>,
    name: S1,
    value: S2,
) -> Result<()>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
    W: Write,
{
    xml_write_element_raw(
        writer,
        name,
        Event::CData(BytesText::from_plain_str(value.as_ref())),
    )
}

pub fn xml_write_element_raw<S1, W>(writer: &mut Writer<W>, name: S1, value: Event) -> Result<()>
where
    S1: AsRef<str>,
    W: Write,
{
    writer.write_event(Event::Start(BytesStart::borrowed_name(
        name.as_ref().as_bytes(),
    )))?;

    writer.write_event(value)?;

    writer.write_event(Event::End(BytesEnd::borrowed(name.as_ref().as_bytes())))?;

    writer.write(b"\n")?;

    Ok(())
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Having {
    pub path: String,
    pub value: Option<Value>,
}

impl Having {
    /// Check if the "Having" matches the provided context.
    pub fn matches(&self, context: &Value) -> Result<bool> {
        Ok(
            match (&self.value, first_value_for_path(context, &self.path)?) {
                (Some(v1), Some(v2)) => v1.eq(v2),
                (None, Some(_)) => true,
                (_, None) => false,
            },
        )
    }
}
