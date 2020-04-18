use env_logger::Env;
use failure::{Error, Fail};

use structopt::StructOpt;

use hagen_core::generator::GeneratorBuilder;

use std::env;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, StructOpt)]
#[structopt(name = "hagen", author = "Jens Reimann")]
pub struct Options {
    /// Override the basename of the site
    #[structopt(short = "b", long = "base")]
    basename: Option<String>,

    /// The root of the site. Must contain the file "hagen.yaml" and the "content" directory.
    #[structopt(short = "r", long = "root")]
    root: Option<String>,

    /// Dump the content files as well.
    #[structopt(short = "D", long = "dump")]
    dump: bool,
}

fn hag_run() -> Result<()> {
    let opts = Options::from_args();

    let root = match opts.root {
        Some(x) => PathBuf::from(x),
        None => env::current_dir().expect("Failed to get current directory"),
    };

    let mut generator = GeneratorBuilder::new(&root)
        .dump(opts.dump)
        .override_basename(opts.basename)
        .build();

    Ok(generator.run()?)
}

/// Exit on error, showing cause of error
fn hag_exit(err: failure::Error) -> ! {
    for cause in Fail::iter_chain(err.as_fail()) {
        eprintln!("{}: {}", cause.name().unwrap_or("Error"), cause);
    }

    std::process::exit(1)
}

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    match hag_run() {
        Err(e) => hag_exit(e),
        Ok(()) => {}
    }
}
