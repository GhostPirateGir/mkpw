extern crate mkpw;
use mkpw::Mkpw;

use std::env;

fn main() {
    let mkpw = Mkpw::new(env::args()).unwrap_or_else(|err| {
        Mkpw::exit();
        eprintln!("Error parsing arguments: {}", err);
        std::process::exit(1);
    });
    mkpw.run();
}
