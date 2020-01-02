use failure::{Error, Fail};
use jsonpath_lib::JsonPathError;

#[derive(Debug, Fail)]
pub enum GeneratorError {
    #[fail(display = "Failed to process")]
    GenericError(#[cause] Error),
    #[fail(display = "JSON path error: {}", _0)]
    JsonPath(String),
}

impl From<JsonPathError> for GeneratorError {
    fn from(err: JsonPathError) -> GeneratorError {
        GeneratorError::JsonPath(err.to_string())
    }
}

impl From<Error> for GeneratorError {
    fn from(err: Error) -> GeneratorError {
        GeneratorError::GenericError(err)
    }
}
