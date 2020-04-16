use failure::Error;
use serde_json::{Map, Value};

type Result<T> = std::result::Result<T, Error>;

use log::debug;

fn is_marker(line: Option<&str>) -> bool {
    if let Some(s) = line {
        s.trim().eq("---")
    } else {
        false
    }
}

pub fn parse_front_matter(data: &String) -> Result<(String, Option<Map<String, Value>>)> {
    let mut lines = data.lines();

    if !is_marker(lines.next()) {
        return Ok((data.clone(), None));
    }

    let mut front_matter: Vec<String> = Vec::new();

    while let Some(s) = lines.next() {
        if is_marker(Some(s)) {
            break;
        }
        front_matter.push(s.into());
    }

    let front_matter = front_matter.join("\n");

    debug!("front matter: {}", front_matter);

    let front_matter = serde_yaml::from_str::<Map<String, Value>>(&front_matter)?;
    let remainder = lines.collect::<Vec<_>>().join("\n");

    debug!("front matter: {:?} -> {}", front_matter, remainder);

    Ok((remainder, Some(front_matter)))
}
