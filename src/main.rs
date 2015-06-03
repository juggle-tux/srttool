#![feature(std_misc)]

extern crate clap;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
pub use srt::*;

pub mod srt;

// print error
macro_rules! printe {
    ($arg:expr) => { println!("ERROR: {:?}", $arg) };
    ($fmt:expr, $($arg:expr)*) => { println!(concat!("ERROR: ", $fmt), $($arg)*) };
}

fn main() {
    let cmd = parse_cmd();
    if let Some(offset) = cmd.value_of("offset") {
        match dur_from_str(offset) {
            Ok(d) => println!("{}", d),
            Err(e) => {
                printe!("{}\noffset must be in the form \"00:11:22,333\" or \"n00:11:22,333\"", e.description());
                return
            }
        }
    }
    match open_file(cmd.value_of("file").expect("")) {
        Ok(lines) => {
            let r = BlockReader::new(lines);
            for b in r {
                println!("{:?}", b);
            }
        }
        Err(e) => printe!(e.to_string()),
    }
}

fn parse_cmd<'a, 'b>() -> clap::ArgMatches<'a, 'b> {
    clap::App::new("srttool")
        .version("0.0.1")
        .author("tux <juggle-tux@users.noreply.github.com>")
        .about("readjust the timing in a srt subtitle file")
        .arg(clap::Arg::with_name("file")
             .index(1)
             .help("The input file")
             .required(true))
        .arg(clap::Arg::with_name("offset")
             .short("o")
             .long("offset")
             .help("The time offset to add. (prfix with n for negative values)")
             .takes_value(true))
        .get_matches()
}

fn open_file(path: &str) -> Result<Lines, io::Error> {
    let file = try!(File::open(path));
    Ok(BufReader::new(file).lines())
}

