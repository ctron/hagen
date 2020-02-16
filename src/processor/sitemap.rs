use crate::processor::{xml_write_element, Processor, ProcessorContext};

use failure::Error;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use relative_path::RelativePath;
use std::fs::File;
use std::io::Write;
use url::Url;

use crate::generator::GeneratorConfig;
use chrono::{DateTime, SecondsFormat, Utc};
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
    last_mod: Option<String>,
    change_frequency: Option<String>,
    priority: Option<String>,
}

pub struct SitemapProcessor;

impl Processor for SitemapProcessor {
    fn create<'a, 'reg>(
        &self,
        _: &'reg mut Handlebars,
        _: &Value,
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
                last_mod.to_rfc3339_opts(SecondsFormat::Secs, true),
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

    fn last_mod_from(
        &self,
        context: &Value,
        handlebars: &Handlebars,
    ) -> Result<Option<DateTime<Utc>>> {
        let last_mod = self
            .config
            .last_mod
            .as_ref()
            .map(|l| value_by_template(context, handlebars, l))
            .transpose()?;

        debug!("last_mod: {:?}", last_mod);

        let last_mod = last_mod
            .filter(|s| !s.is_empty())
            .map(|t| DateTime::parse_from_rfc3339(&t))
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
        let last_mod = self.last_mod_from(context, handlebars)?;

        // change freq

        let change_freq = self
            .config
            .change_frequency
            .as_ref()
            .map(|c| value_by_template(context, handlebars, c))
            .transpose()?
            .filter(|s| !s.is_empty());

        let change_freq = match change_freq {
            Some(s) => Some(ChangeFrequency::from_str(&s)?),
            _ => None,
        };

        // priority

        let priority = self
            .config
            .priority
            .as_ref()
            .map(|p| value_by_template(context, handlebars, p))
            .transpose()?
            .filter(|s| !s.is_empty());

        let priority = match priority {
            Some(s) => Some(s.parse::<f64>().map_err(|err| {
                GeneratorError::GenericDetailError(
                    err.into(),
                    format!("'priority' must be a numeric value, or unset: {}", s),
                )
            })?),
            _ => None,
        };

        // write entry

        self.write_entry(&url, last_mod, change_freq, priority)?;

        // done

        Ok(())
    }

    fn complete(&mut self, _: &mut Handlebars) -> Result<()> {
        // close xml tag
        self.writer
            .write_event(Event::End(BytesEnd::borrowed(b"urlset")))?;

        Ok(())
    }
}

fn value_by_template(context: &Value, handlebars: &Handlebars, template: &str) -> Result<String> {
    handlebars
        .render_template(template, context)
        .map_err(|err| GeneratorError::TemplateRenderError(err).into())
        .map(|s| s.trim().to_string())
}
