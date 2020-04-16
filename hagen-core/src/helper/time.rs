use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
};

use chrono::{DateTime, Utc};

#[derive(Clone, Copy)]
pub struct TimeHelper;

impl HelperDef for TimeHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let format = h
            .param(0)
            .map(|p| p.value())
            .ok_or(RenderError::new("Missing format parameter"))?
            .as_str()
            .ok_or(RenderError::new("Format must be a string"))?;

        let value = h.param(1).map(|p| p.value());

        let value = if let Some(timestamp) = value {
            match timestamp.as_str() {
                Some(s) => DateTime::parse_from_rfc3339(s)
                    .map_err(|err| RenderError::with(err))?
                    .with_timezone(&Utc),
                _ => {
                    return Err(RenderError::new(format!(
                        "Timestamp is not a string: {:?}",
                        timestamp
                    )))
                }
            }
        } else {
            Utc::now()
        };

        let result = value.format(format);

        let result = result.to_string();
        out.write(&result)?;

        Ok(())
    }
}
