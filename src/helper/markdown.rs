use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, Output, RenderContext, RenderError,
};
use pulldown_cmark::{html, Options, Parser};

#[derive(Clone, Copy)]
pub struct MarkdownifyHelper;

impl HelperDef for MarkdownifyHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _: &'reg Handlebars,
        _: &'rc Context,
        _: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let markdown_input = h
            .param(0)
            .ok_or(RenderError::new("Missing content for markdownify"))?;

        if let Some(markdown_input) = markdown_input.value().as_str() {
            let mut options = Options::empty();
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TABLES);
            let parser = Parser::new_ext(markdown_input, options);

            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);

            out.write(&html_output)?;

            Ok(())
        } else {
            Err(RenderError::new("Require string data for markdownify"))
        }
    }
}
