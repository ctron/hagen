use serde_json::Value;

use crate::error::GeneratorError;
use failure::Error;

type Result<T> = std::result::Result<T, Error>;

/// Wrapper for working with jsonpath.
pub fn values_for_path<'a>(context: &'a Value, path: &'a str) -> Result<Vec<&'a Value>> {
    Ok(jsonpath_lib::select(context, path)
        .map_err(|err| GeneratorError::JsonPath(err.to_string()))?)
}

/// Get the first value of a jsonpath query.
pub fn first_value_for_path<'a>(context: &'a Value, path: &'a str) -> Result<Option<&'a Value>> {
    let mut v = values_for_path(context, path)?;
    Ok(v.pop())
}
