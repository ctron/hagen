use crate::generator::GeneratorConfig;
use crate::helper::url::full_url_for;
use crate::processor::{xml_write_element, Processor, ProcessorContext};
use chrono::Utc;
use failure::Error;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use relative_path::RelativePath;
use serde_json::Value;
use std::fs::File;
use std::io::Write;

type Result<T> = std::result::Result<T, Error>;

pub struct RssProcessor {}

impl RssProcessor {
    pub fn new() -> Self {
        RssProcessor {}
    }
}

impl Processor for RssProcessor {
    fn create<'a>(&self, config: &'a GeneratorConfig) -> Result<Box<dyn ProcessorContext + 'a>> {
        let writer = File::create(config.output.join("feed.rss"))?;
        let mut writer = Writer::new(writer);

        writer.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;
        writer.write(b"\n")?;
        writer.write_event(Event::Start(
            BytesStart::borrowed_name(b"rss").with_attributes(vec![("version", "2.0")].into_iter()),
        ))?;
        writer.write(b"\n")?;

        writer.write_event(Event::Start(BytesStart::borrowed_name(b"channel")))?;
        writer.write(b"\n")?;

        xml_write_element(&mut writer, "link", full_url_for(&config.basename, "/")?)?;

        let now = Utc::now();
        xml_write_element(&mut writer, "lastBuildDate", now.to_rfc2822().to_string())?;

        Ok(Box::new(RssContext::<'a> { writer, config }))
    }
}

pub struct RssContext<'a, W: Write> {
    writer: Writer<W>,
    config: &'a GeneratorConfig,
}

impl<'a, W: Write> ProcessorContext for RssContext<'a, W> {
    fn file_created(&mut self, path: &RelativePath, context: &Value) -> Result<()> {
        Ok(())
    }

    fn complete(&mut self) -> Result<()> {
        self.writer
            .write_event(Event::End(BytesEnd::borrowed(b"channel")))?;
        self.writer.write(b"\n")?;

        self.writer
            .write_event(Event::End(BytesEnd::borrowed(b"rss")))?;
        self.writer.write(b"\n")?;

        Ok(())
    }
}
