/*
Copyright 2015 juggle-tux

This file is part of srttool.

srttool is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

Foobar is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with srttool.  If not, see <http://www.gnu.org/licenses/>.
*/

#![cfg(not(test))]
extern crate clap;
extern crate srt;

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::ops::{Add, Sub};
use std::str::FromStr;
use clap::{App, Arg, ArgMatches};
use srt::{Block, BlockReader, Time, Times};

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

macro_rules! trye {
    ($expr:expr) => { match $expr {
        Ok(res) => res,
        Err(e) => return printe!(e.to_string()),
    }}
}

fn main() {
    let cmd = parse_cmd();

    let (offset, neg) =
        if let Some(o) = cmd.value_of("offset") {
            if o.starts_with('n') {
                let d = trye!(Time::from_str(o.trim_left_matches('n')));
                (Times::from(d), true)
            } else {
                (Times::from(trye!(Time::from_str(o))), false)
            }
        } else { (Times::new(), false) };
    let add_offset_to =
        |b: &Block| -> Times {
            if neg {
                b.times.sub(offset)
            } else {
                b.times.add(offset)
            }
        };
    let mut outfile: BufWriter<Box<Write>> =
        if let Some(p) = cmd.value_of("outfile") {
            BufWriter::new(Box::new(trye!(File::create(p))))
        } else {
            BufWriter::new(Box::new(io::stdout()))
        };
    
    let mut i = 0;
    for path in cmd.values_of("infile").expect("Input file is required") {
        let mut infile = BlockReader::new(BufReader::new(trye!(File::open(path))));
        while let Some(b) = infile.next() {
            let mut b = trye!(b);
            b.times = add_offset_to(&b);
            i += 1;
            trye!(write!(&mut outfile, "{}\n{}", i, b))
        }
        println_stderr!("from \"{}\" {} lines parsed", path, infile.line)
    }
    trye!(outfile.flush())
}


fn parse_cmd<'a, 'b>() -> ArgMatches<'a, 'b> {
    App::new("srttool")
        .version("0.0.1")
        .author("Juggle Tux <juggle-tux@users.noreply.github.com>")
        .about("readjust the timing in a srt subtitle file")
        .arg(Arg::with_name("infile")
             .index(1)
             .help("The input files")
             .required(true)
             .multiple(true))
        .arg(Arg::with_name("offset")
             .short("o")
             .long("offset")
             .help("The time offset to add. (prfix with n for negative values)")
             .takes_value(true))
        .arg(Arg::with_name("outfile")
             .short("f")
             .long("out-file")
             .help("Output file default: stdout")
             .takes_value(true))
        .get_matches()
}
