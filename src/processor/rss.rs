use crate::generator::GeneratorConfig;
use crate::helper::url::full_url_for;
use crate::processor::{xml_write_element, Processor, ProcessorContext};
use chrono::{DateTime, Utc};
use failure::Error;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use relative_path::RelativePath;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::GeneratorError;
use crate::path::first_value_for_path;
use handlebars::Handlebars;
use std::fs::File;
use std::io::Write;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RssProcessorConfig {
    site: Site,
    pages: Vec<Page>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
struct Site {
    pub title: Option<String>,
    pub language: Option<String>,
    pub description: Option<String>,
    pub update_period: String,
    pub update_frequency: u32,
    pub update_base: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Page {
    pub published: Option<String>,
    pub updated: Option<String>,
    pub having: Having,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Having {
    pub path: String,
    pub value: Option<Value>,
}

impl Default for Site {
    fn default() -> Self {
        Site {
            title: None,
            language: None,
            description: None,
            update_period: "hourly".into(),
            update_frequency: 1,
            update_base: None,
        }
    }
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

pub struct RssProcessor;

impl RssProcessor {}

impl Processor for RssProcessor {
    fn create<'a>(
        &self,
        handlebars: &mut Handlebars,
        data: &Value,
        generator_config: &'a GeneratorConfig,
        processor_config: Value,
    ) -> Result<Box<dyn ProcessorContext + 'a>> {
        let config: RssProcessorConfig = serde_json::from_value(processor_config)?;

        let writer = File::create(generator_config.output.join("feed.rss"))?;
        let mut writer = Writer::new(writer);

        writer.write_event(Event::Decl(BytesDecl::new(b"1.0", Some(b"UTF-8"), None)))?;
        writer.write(b"\n")?;
        writer.write_event(Event::Start(
            BytesStart::borrowed_name(b"rss").with_attributes(
                vec![
                    ("version", "2.0"),
                    ("xmlns:atom", "http://www.w3.org/2005/Atom"),
                    ("xmlns:content", "http://purl.org/rss/1.0/modules/content/"),
                    ("xmlns:dc", "http://purl.org/dc/elements/1.1/"),
                    ("xmlns:sy", "http://purl.org/rss/1.0/modules/syndication/"),
                ]
                .into_iter(),
            ),
        ))?;
        writer.write(b"\n")?;

        writer.write_event(Event::Start(BytesStart::borrowed_name(b"channel")))?;
        writer.write(b"\n")?;

        // link

        xml_write_element(
            &mut writer,
            "link",
            full_url_for(&generator_config.basename, "/")?,
        )?;

        // atom:link

        let feed_url = full_url_for(&generator_config.basename, "/feed.rss")?;
        writer.write_event(Event::Empty(
            BytesStart::borrowed_name(b"atom:link").with_attributes(
                vec![
                    ("href", feed_url.as_str()),
                    ("rel", "self"),
                    ("type", "application/rss+xml"),
                ]
                .into_iter(),
            ),
        ))?;
        writer.write(b"\n")?;

        // last build date

        let now = Utc::now();
        xml_write_element(&mut writer, "lastBuildDate", now.to_rfc2822().to_string())?;

        // generator

        xml_write_element(&mut writer, "generator", "https://github.com/ctron/hagen")?;

        // site

        if let Some(ref title) = config.site.title {
            let title = handlebars.render_template(title.as_str(), data)?;
            xml_write_element(&mut writer, "title", title)?;
        }
        if let Some(ref language) = config.site.language {
            let title = handlebars.render_template(language.as_str(), data)?;
            xml_write_element(&mut writer, "language", title)?;
        }
        if let Some(ref description) = config.site.description {
            let title = handlebars.render_template(description.as_str(), data)?;
            xml_write_element(&mut writer, "description", title)?;
        }

        // sy

        xml_write_element(&mut writer, "sy:updatePeriod", &config.site.update_period)?;
        xml_write_element(
            &mut writer,
            "sy:updateFrequency",
            format!("{:.2}", &config.site.update_frequency),
        )?;
        if let Some(ref update_base) = config.site.update_base {
            xml_write_element(&mut writer, "sy:updateBase", update_base)?;
        }

        Ok(Box::new(RssContext::<'a> {
            config,
            writer,
            generator_config,
        }))
    }
}

pub struct RssContext<'a, W: Write> {
    config: RssProcessorConfig,
    writer: Writer<W>,
    generator_config: &'a GeneratorConfig,
}

fn fetch_timestamp(
    page_path: &RelativePath,
    context: &Value,
    path: &str,
) -> Result<Option<DateTime<Utc>>> {
    let x = first_value_for_path(context, path)?
        .and_then(|s| s.as_str())
        .map(|s| DateTime::parse_from_rfc3339(s))
        .transpose()?
        .map(|d| d.with_timezone(&Utc));

    Ok(x)
}

impl<'a, W: Write> RssContext<'a, W> {
    fn matches(
        &self,
        context: &Value,
        path: &RelativePath,
    ) -> Result<Option<(Option<DateTime<Utc>>, Option<DateTime<Utc>>)>> {
        for p in &self.config.pages {
            if p.having.matches(context)? {
                let published = p
                    .published
                    .as_ref()
                    .map(|s| fetch_timestamp(path, context, &s))
                    .transpose()?;

                let published: Option<DateTime<Utc>> = published.ok_or_else(|| {
                    GeneratorError::Error(format!(
                        "Missing value '{}' for RSS page {}",
                        p.published.as_ref().unwrap(),
                        path
                    ))
                })?;

                let updated = p
                    .updated
                    .as_ref()
                    .map(|s| fetch_timestamp(path, context, &s))
                    .transpose()?
                    .unwrap_or(published);

                return Ok(Some((published, updated)));
            }
        }

        Ok(None)
    }
}

impl<'a, W: Write> ProcessorContext for RssContext<'a, W> {
    fn file_created(
        &mut self,
        path: &RelativePath,
        context: &Value,
        handlebars: &mut Handlebars,
    ) -> Result<()> {
        let m = self.matches(context, path)?;
        if m.is_none() {
            return Ok(());
        }

        let m = m.unwrap();

        // item

        self.writer
            .write_event(Event::Start(BytesStart::borrowed_name(b"item")))?;
        self.writer.write(b"\n")?;

        // link

        let url = crate::helper::url::full_url_for(&self.generator_config.basename, path.as_str())?;
        self.writer.write(b"\t")?;
        xml_write_element(&mut self.writer, "link", &url)?;

        // guid

        self.writer.write(b"\t")?;
        xml_write_element(&mut self.writer, "guid", &url)?;

        // pubDate

        if let Some(published) = m.0 {
            self.writer.write(b"\t")?;
            xml_write_element(&mut self.writer, "pubDate", published.to_rfc2822())?;
        }

        // /item

        self.writer
            .write_event(Event::End(BytesEnd::borrowed(b"item")))?;
        self.writer.write(b"\n")?;

        Ok(())
    }

    fn complete(&mut self, handlebars: &mut Handlebars) -> Result<()> {
        self.writer
            .write_event(Event::End(BytesEnd::borrowed(b"channel")))?;
        self.writer.write(b"\n")?;

        self.writer
            .write_event(Event::End(BytesEnd::borrowed(b"rss")))?;
        self.writer.write(b"\n")?;

        Ok(())
    }
}