// Copyright 2015 juggle-tux
//
// This file is part of srttool.
//
// srttool is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Foobar is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with srttool.  If not, see <http://www.gnu.org/licenses/>.
//

#![cfg(not(test))]
extern crate clap;
extern crate srt;

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::str::FromStr;
use clap::{App, Arg, ArgMatches};
use srt::{BlockReader, Time};

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

    let (offset, neg) = if let Some(o) = cmd.value_of("offset") {
        if o.starts_with('n') {
            (trye!(Time::from_str(o.trim_left_matches('n'))), true)
        } else {
            (trye!(Time::from_str(o)), false)
        }
    } else {
        (Time::default(), false)
    };
    let add_offset_to = |b| {
        if neg {
            b - offset
        } else {
            b + offset
        }
    };

    let mut outfile: BufWriter<Box<Write>> = if let Some(p) = cmd.value_of("outfile") {
        BufWriter::new(Box::new(trye!(File::create(p))))
    } else {
        BufWriter::new(Box::new(io::stdout()))
    };

    let mut i = 0;
    for path in cmd.values_of("infile").expect("Input file is required") {
        let mut dec = BlockReader::new(BufReader::new(trye!(File::open(path))));
        for b in dec.by_ref() {
            i += 1;
            trye!(write!(&mut outfile, "{}\n{}", i, add_offset_to(trye!(b))))
        }
        println_stderr!("{} lines in {:?} parsed", dec.line(), path)
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
