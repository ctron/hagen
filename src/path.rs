use serde_json::Value;

use crate::error::GeneratorError;
use failure::Error;

type Result<T> = std::result::Result<T, Error>;

use log::info;

pub fn values_for_path<'a>(context: &'a Value, path: &'a str) -> Result<Vec<&'a Value>> {
    let v = jsonpath_lib::select(context, path)
        .map_err(|err| GeneratorError::JsonPath(err.to_string()))?;

    info!("Result: {:?}", v);

    Ok(v)
}

pub fn first_value_for_path<'a>(context: &'a Value, path: &'a str) -> Result<Option<&'a Value>> {
    let mut v = values_for_path(context, path)?;
    let v = v.pop();

    info!("Result: {:?}", v);

    Ok(v)
}
