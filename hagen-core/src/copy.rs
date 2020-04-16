use failure::Error;
use std::fs;
use std::path::Path;

use globset::{Glob, GlobSetBuilder};
use log::debug;
use walkdir::WalkDir;

type Result<T> = std::result::Result<T, Error>;

pub fn copy_dir<P1, P2, S>(from: P1, to: P2, glob: Option<S>) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
    S: AsRef<str>,
{
    let from = from.as_ref();
    let to = to.as_ref();

    let mut builder = GlobSetBuilder::new();
    if let Some(ref g) = glob {
        builder.add(Glob::new(g.as_ref())?);
    }
    let set = builder.build()?;

    for item in WalkDir::new(&from).contents_first(false).follow_links(true) {
        debug!("Found: {:?}", item);
        match item {
            Ok(entry) => {
                let source = entry.path();
                let relative = source.strip_prefix(from)?;

                if set.is_empty() || set.is_match(relative) {
                    let target = to.join(relative);

                    if source.is_file() {
                        debug!("Copy file - to: {:?}", target);
                        if let Some(parent) = target.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::copy(source, target)?;
                    } else if source.is_dir() {
                        debug!("Create directory: {:?}", target);
                        fs::create_dir_all(target)?;
                    }
                }
            }
            Err(_) => {}
        }
    }

    Ok(())
}
