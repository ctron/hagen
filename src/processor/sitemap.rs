use crate::processor::{xml_write_element, Processor, ProcessorContext};

use failure::Error;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use relative_path::RelativePath;
use std::fs::File;
use std::io::Write;
use url::Url;

use crate::generator::GeneratorConfig;
use chrono::{DateTime, Utc};
use std::str::FromStr;
use strum_macros::{AsRefStr, AsStaticStr, EnumString};

use crate::error::GeneratorError;
use serde_json::Value;

use log::debug;

use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SitemapProcessorConfig {
    published_path: String,
    updated_path: String,
}

pub struct SitemapProcessor;

impl Processor for SitemapProcessor {
    fn create<'a, 'reg>(
        &self,
        handlebars: &'reg mut Handlebars,
        data: &Value,
        generator_config: &'a GeneratorConfig,
        processor_config: Value,
    ) -> Result<Box<dyn ProcessorContext + 'a>> {
        let config = serde_json::from_value(processor_config)?;

        let writer = File::create(generator_config.output.join("sitemap.xml"))?;
        let mut writer = Writer::new(writer);

        writer.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;
        writer.write(b"\n")?;
        writer.write_event(Event::Start(
            BytesStart::borrowed_name(b"urlset").with_attributes(
                vec![("xmlns", "http://www.sitemaps.org/schemas/sitemap/0.9")].into_iter(),
            ),
        ))?;
        writer.write(b"\n")?;

        Ok(Box::new(SitemapContext::<'a> {
            config,
            writer,
            generator_config,
        }))
    }
}

pub struct SitemapContext<'a, W: Write> {
    config: SitemapProcessorConfig,

    writer: Writer<W>,
    generator_config: &'a GeneratorConfig,
}

#[derive(AsRefStr, AsStaticStr, EnumString)]
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
    fn write_entry(
        &mut self,
        loc: &Url,
        last_mod: Option<DateTime<Utc>>,
        change_freq: Option<ChangeFrequency>,
        priority: Option<f64>,
    ) -> Result<()> {
        self.writer
            .write_event(Event::Start(BytesStart::borrowed_name(b"url")))?;
        self.writer.write(b"\n")?;

        self.writer.write(b"\t")?;
        xml_write_element(&mut self.writer, "loc", &loc)?;
        if let Some(last_mod) = last_mod {
            self.writer.write(b"\t")?;
            xml_write_element(
                &mut self.writer,
                "lastmod",
                last_mod.format("%Y-%m-%d").to_string(),
            )?;
        }
        if let Some(change_freq) = change_freq {
            self.writer.write(b"\t")?;
            xml_write_element(&mut self.writer, "changefreq", &change_freq)?;
        }
        if let Some(priority) = priority {
            self.writer.write(b"\t")?;
            xml_write_element(&mut self.writer, "priority", format!("{:.2}", priority))?;
        }

        self.writer
            .write_event(Event::End(BytesEnd::borrowed(b"url")))?;
        self.writer.write(b"\n")?;
        Ok(())
    }

    fn last_mod_from(&self, context: &Value) -> Result<Option<DateTime<Utc>>> {
        let published = value_by_path(context, &self.config.published_path)?;
        let updated = value_by_path(context, &self.config.updated_path)?;

        debug!("published: {:?}, updated: {:?}", published, updated);

        let last_mod = match (published.as_slice(), updated.as_slice()) {
            (_, [t]) => Some(t),
            ([t], []) => Some(t),
            _ => None,
        }
        .and_then(|t| t.as_str())
        .map(|t| DateTime::parse_from_rfc3339(t))
        .transpose()?
        .map(|t| t.with_timezone(&Utc));

        Ok(last_mod)
    }
}

impl<'a, W: Write> ProcessorContext for SitemapContext<'a, W> {
    fn file_created(
        &mut self,
        path: &RelativePath,
        context: &Value,
        handlebars: &mut Handlebars,
    ) -> Result<()> {
        let url = crate::helper::url::full_url_for(&self.generator_config.basename, path.as_str())?;
        let last_mod = self.last_mod_from(context)?;

        // change freq

        let change_freq = value_by_path(
            context,
            "$.context.page.frontMatter.sitemap.changeFrequency",
        )?
        .first()
        .and_then(|s| s.as_str());

        let change_freq = match change_freq {
            Some(s) => Some(ChangeFrequency::from_str(s)?),
            _ => None,
        };

        // priority

        let priority = value_by_path(context, "$.context.page.frontMatter.sitemap.priority")?;

        let priority = match priority.first() {
            Some(s) => Some(s.as_f64().ok_or(GeneratorError::Error(
                "'priority' must be a numeric value, or unset".into(),
            ))?),
            _ => None,
        };

        self.write_entry(&url, last_mod, change_freq, priority)?;

        Ok(())
    }

    fn complete(&mut self, handlebars: &mut Handlebars) -> Result<()> {
        self.writer
            .write_event(Event::End(BytesEnd::borrowed(b"urlset")))?;

        Ok(())
    }
}

fn value_by_path<'a>(context: &'a Value, path: &'a str) -> Result<Vec<&'a Value>> {
    let result = jsonpath_lib::select(context, path.as_ref())
        .map_err(|e| GeneratorError::JsonPath(e.to_string()))?;

    Ok(result)
}
