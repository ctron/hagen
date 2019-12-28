extern crate handlebars;

mod error;
mod generator;

use std::env;
use std::path::Path;

use generator::Generator;

fn hag_exit(err: failure::Error) -> ! {
    println!("{}", err);
    std::process::exit(1)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let generator = Generator::new(Path::new(args.get(1).expect("Missing directory")));
    let result = generator.run();

    match result {
        Err(e) => hag_exit(e),
        Ok(()) => {}
    }
}
