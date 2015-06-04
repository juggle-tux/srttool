extern crate clap;
extern crate srttool;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use srttool::*;

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

macro_rules! printe {
    ($arg:expr) => { println_stderr!("ERROR: {:?}", $arg) };
    ($fmt:expr, $($arg:expr),*) => { println_stderr!(concat!("ERROR: ", $fmt), $($arg),*) };
}

fn main() {
    let cmd = parse_cmd();
    let offset = if let Some(o) = cmd.value_of("offset") {
        match dur_from_str(o) {
            Ok(d) => Times::from(&d),
            Err(e) => {
                printe!("{}\noffset must be in the form \"00:11:22,333\" or \"n00:11:22,333\"", e.description());
                return;
            }
        }
    } else { Times::new() };

    
    match open_file(cmd.value_of("file").expect("")) {
        Ok(lines) => {
            let mut r = BlockReader::new(lines);
            let mut i = 0;

            while let Some(mut b) = r.next() {
                b.times = b.times + offset;
                i += 1;
                print!("{}\n{}", i, b);
            }

            let l = r.line;
            match r.err() {
                None => println_stderr!("Finish after {} lines", l),
                Some(e) => printe!("Line {}: {:?}", l, e),
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

