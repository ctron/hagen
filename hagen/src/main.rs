use env_logger::Env;
use failure::{Error, Fail};

use structopt::StructOpt;

use hagen_core::generator::Generator;
use hagen_core::generator::Options;

type Result<T> = std::result::Result<T, Error>;

fn hag_exit(err: failure::Error) -> ! {
    for cause in Fail::iter_chain(err.as_fail()) {
        println!("{}: {}", cause.name().unwrap_or("Error"), cause);
    }

    std::process::exit(1)
}

fn hag_run() -> Result<()> {
    let opts = Options::from_args();

    let mut generator = Generator::new(opts);
    Ok(generator.run()?)
}

fn main() {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    match hag_run() {
        Err(e) => hag_exit(e),
        Ok(()) => {}
    }
}
