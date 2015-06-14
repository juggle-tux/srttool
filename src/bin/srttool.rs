extern crate clap;
extern crate srt;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use srt::{BlockReader, Times};

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

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
            Err(e) => {
                printe!("{}\noffset must be in the form \"00:11:22,333\" or \"n00:11:22,333\"", e.description());
                return;
            }
        }
    } else { Times::new() };

    match open_file(cmd.value_of("file").expect("")) {
        Err(e) => printe!(e.to_string()),
        Ok(file) => {
            let mut r = BlockReader::new(file);
            let mut i = 0;
            let mut of: Box<io::Write> = match cmd.value_of("outfile") {
                Some(p) => Box::new(File::create(p).unwrap()),
                None => Box::new(io::stdout()),
            };
            
            while let Some(b) = r.next() {
                match b {
                    Ok(mut b) => {
                        b.times = b.times + offset;
                        i += 1;
                        if let Err(e) = write!(&mut of, "{}\n{}", i, b) {
                            printe!(e);
                            return;
                        }
                    }
                    Err(e) => printe!(e),
                }
            }
            println_stderr!("{} lines done", r.line_nr())          
        }
    }
}

fn parse_cmd<'a, 'b>() -> clap::ArgMatches<'a, 'b> {
    clap::App::new("srttool")
        .version("0.0.1")
        .author("Juggle Tux <juggle-tux@users.noreply.github.com>")
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
