use crate::processor::{Processor, ProcessorContext};

use failure::Error;
use failure::_core::str::FromStr;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use relative_path::RelativePath;
use std::fs::File;
use std::io::Write;
use url::Url;

use crate::generator::GeneratorContext;
use chrono::{DateTime, Utc};
use strum_macros::{AsRefStr, AsStaticStr};

use chrono::TimeZone;

type Result<T> = std::result::Result<T, Error>;

pub struct SitemapProcessor {}

impl SitemapProcessor {
    pub fn new() -> SitemapProcessor {
        SitemapProcessor {}
    }
}

impl Processor for SitemapProcessor {
    fn create<'a>(&self, context: &'a GeneratorContext) -> Result<Box<dyn ProcessorContext + 'a>> {
        let writer = File::create(context.output.join("sitemap.xml"))?;
        let mut writer = Writer::new(writer);

        writer.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;
        writer.write(b"\n")?;
        writer.write_event(Event::Start(
            BytesStart::borrowed_name(b"urlset").with_attributes(
                vec![("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9")].into_iter(),
            ),
        ))?;
        writer.write(b"\n")?;

        Ok(Box::new(SitemapContext::<'a> { writer, context }))
    }
}

pub struct SitemapContext<'a, W: Write> {
    writer: Writer<W>,
    context: &'a GeneratorContext<'a>,
}

#[derive(AsRefStr, AsStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum ChangeFrequency {
    Always,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Never,
}

impl<'a, W: Write> SitemapContext<'a, W> {
    fn write_element<S1: AsRef<str>, S2: AsRef<str>>(&mut self, name: S1, value: S2) -> Result<()> {
        self.writer
            .write_event(Event::Start(BytesStart::borrowed_name(
                name.as_ref().as_bytes(),
            )))?;

        self.writer
            .write_event(Event::Text(BytesText::from_plain_str(value.as_ref())))?;

        self.writer
            .write_event(Event::End(BytesEnd::borrowed(name.as_ref().as_bytes())))?;

        self.writer.write(b"\n")?;

        Ok(())
    }

    fn write_entry(
        &mut self,
        loc: &Url,
        last_mod: Option<DateTime<Utc>>,
        change_freq: Option<ChangeFrequency>,
        priority: Option<f32>,
    ) -> Result<()> {
        self.writer
            .write_event(Event::Start(BytesStart::borrowed_name(b"url")))?;
        self.writer.write(b"\n")?;

        self.writer.write(b"\t")?;
        self.write_element("loc", &loc)?;
        if let Some(last_mod) = last_mod {
            self.writer.write(b"\t")?;
            self.write_element("lastmod", last_mod.format("%Y-%m-%d").to_string())?;
        }
        if let Some(change_freq) = change_freq {
            self.writer.write(b"\t")?;
            self.write_element("changefreq", &change_freq)?;
        }
        if let Some(priority) = priority {
            self.writer.write(b"\t")?;
            self.write_element("priority", format!("{:.2}", priority))?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::borrowed(b"url")))?;
        self.writer.write(b"\n")?;
        Ok(())
    }
}

impl<'a, W: Write> ProcessorContext for SitemapContext<'a, W> {
    fn file_created(&mut self, path: &RelativePath) -> Result<()> {
        let url = crate::helper::url::full_url_for(self.context.basename, path.as_str())?;

        self.write_entry(&url, None, None, None)?;

        Ok(())
    }

    fn complete(&mut self) -> Result<()> {
        self.writer
            .write_event(Event::End(BytesEnd::borrowed(b"urlset")))?;

        Ok(())
    }
}
