use failure::{Error, Fail};
use handlebars::{RenderError, TemplateFileError};
use jsonpath_lib::JsonPathError;
use url::ParseError;

#[derive(Debug, Fail)]
pub enum GeneratorError {
    #[fail(display = "Failed to process")]
    GenericError(#[cause] Error),
    #[fail(display = "Failed to process")]
    JsonError(#[cause] serde_json::Error),
    #[fail(display = "Failed to process")]
    YamlError(#[cause] serde_yaml::Error),
    #[fail(display = "Failed to process")]
    TemplateError(#[cause] TemplateFileError),
    #[fail(display = "Failed to process")]
    IoError(#[cause] std::io::Error),
    #[fail(display = "{}", _0)]
    Error(String),
    #[fail(display = "JSON path error: {}", _0)]
    JsonPath(String),
}

impl From<GeneratorError> for RenderError {
    fn from(err: GeneratorError) -> Self {
        RenderError::with(err.compat())
    }
}

impl From<TemplateFileError> for GeneratorError {
    fn from(err: TemplateFileError) -> GeneratorError {
        GeneratorError::TemplateError(err)
    }
}

impl From<std::io::Error> for GeneratorError {
    fn from(err: std::io::Error) -> GeneratorError {
        GeneratorError::IoError(err)
    }
}

impl From<serde_json::Error> for GeneratorError {
    fn from(err: serde_json::Error) -> GeneratorError {
        GeneratorError::JsonError(err)
    }
}

impl From<serde_yaml::Error> for GeneratorError {
    fn from(err: serde_yaml::Error) -> GeneratorError {
        GeneratorError::YamlError(err)
    }
}

impl From<JsonPathError> for GeneratorError {
    fn from(err: JsonPathError) -> GeneratorError {
        GeneratorError::JsonPath(err.to_string())
    }
}

impl From<handlebars::TemplateRenderError> for GeneratorError {
    fn from(err: handlebars::TemplateRenderError) -> Self {
        GeneratorError::GenericError(err.into())
    }
}

impl From<handlebars::RenderError> for GeneratorError {
    fn from(err: handlebars::RenderError) -> Self {
        GeneratorError::GenericError(err.into())
    }
}

impl From<ParseError> for GeneratorError {
    fn from(err: ParseError) -> Self {
        GeneratorError::GenericError(err.into())
    }
}

impl From<Error> for GeneratorError {
    fn from(err: Error) -> GeneratorError {
        GeneratorError::GenericError(err)
    }
}
