#![cfg(not(test))]
extern crate clap;
extern crate srt;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};
use srt::{BlockReader, Times};

macro_rules! println_stderr{
    ($($arg:tt)*) => {
        match writeln!(&mut io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    }
}

macro_rules! printe {
    ($arg:expr) => {
        println_stderr!("ERROR: {:?}", $arg)
    };
    ($fmt:expr, $($arg:expr),*) => {
        println_stderr!(concat!("ERROR: ", $fmt), $($arg),*)
    };
}

fn main() {
    let cmd = parse_cmd();
    let offset = if let Some(o) = cmd.value_of("offset") {
        match srt::dur_from_str(o) {
            Ok(d) => Times::from(&d),
            Err(e) => return printe!("{}\noffset must be in the form \"00:11:22,333\" or \"n00:11:22,333\""
                                     , e.description()),
        }
    } else { Times::new() };
    
    let mut outfile: Box<io::Write> = if let Some(p) = cmd.value_of("outfile") {
        match File::create(p) {
            Ok(f) => Box::new(BufWriter::new(f)),
            Err(e) => return printe!(e.to_string()),
        }
    } else { Box::new(io::stdout()) };
    
    let mut i = 0;
    for path in cmd.values_of("infile").expect("Input file is required") {
        let mut infile = match open_file(path) {
            Ok(f) => BlockReader::new(f),
            Err(e) => return printe!(e.to_string()),
        };
        
        while let Some(b) = infile.next() {
            match b {
                Ok(mut b) => {
                    b.times = b.times + offset;
                    i += 1;
                    if let Err(e) = write!(&mut outfile, "{}\n{}", i, b) {
                        return printe!(e.to_string())
                    }
                }
                Err(e) => return printe!(e.to_string()),
            }
        }
        println_stderr!("from \"{}\" {} lines parsed", path, infile.line)
    }
    if let Err(e) = outfile.flush() { return printe!(e.to_string()) }
}


fn parse_cmd<'a, 'b>() -> clap::ArgMatches<'a, 'b> {
    clap::App::new("srttool")
        .version("0.0.1")
        .author("Juggle Tux <juggle-tux@users.noreply.github.com>")
        .about("readjust the timing in a srt subtitle file")
        .arg(clap::Arg::with_name("infile")
             .index(1)
             .help("The input files")
             .required(true)
             .multiple(true))
        .arg(clap::Arg::with_name("offset")
             .short("o")
             .long("offset")
             .help("The time offset to add. (prfix with n for negative values)")
             .takes_value(true))
        .arg(clap::Arg::with_name("outfile")
             .short("f")
             .long("out-file")
             .help("Output file default: stdout")
             .takes_value(true))
        .get_matches()
}

fn open_file(path: &str) -> Result<BufReader<File>, io::Error> {
    let file = try!(File::open(path));
    Ok(BufReader::new(file))
}
