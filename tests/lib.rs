/*
Copyright 2015 juggle-tux

This file is part of srttool.

Foobar is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

Foobar is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with Foobar.  If not, see <http://www.gnu.org/licenses/>.
*/
#![cfg(test)]

extern crate srt;

use std::io::Cursor;
use srt::*;
use std::time::Duration;
use std::str::FromStr;

const BLOCK: &'static [u8] = b"1\n00:00:22,280 --> 00:00:34,090\nNORIKO'S DINNER TABLE\n\n";

#[test]
fn parse_time() {
    let d = match Time::from_str("01:02:03,456") {
        Ok(d) => {
            assert_eq!(Duration::new(3723, 456_000_000), Duration::from(d));
            d
        }
        Err(e) => panic!(e),
    };
    let mut t = StartEnd::from(d);
    t.1 = t.1 + Time::from_str("6:5:4,321").unwrap();
    assert_eq!(format!("{}",t), "01:02:03,456 --> 07:07:07,777");

    t = StartEnd::new() - d;
    assert_eq!(format!("{}", t), "00:00:00,000 --> 00:00:00,000");
}

#[test]
fn parse_block() {
    match BlockReader::new(Cursor::new(BLOCK)).next() {
        Some(b) => match b {
            Ok(b) => assert_eq!("00:00:22,280 --> 00:00:34,090\nNORIKO\'S DINNER TABLE\n\n", format!("{}", b)),
            Err(e) => panic!(e),
        },
        None => panic!("didn't got any block"),
    }
}
