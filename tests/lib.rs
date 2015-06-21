#![feature(std_misc)] #![feature(std_misc)]
extern crate srt;

//use std::io::prelude::*;
use std::io::{Cursor};
use srt::*;
use std::time::Duration;
use std::ops::{Add, Mul};

const BLOCK: &'static [u8] = b"1\n00:00:22,280 --> 00:00:34,090\nNORIKO'S DINNER TABLE\n\n";
const TIME: &'static str = "01:02:03,456";
#[test]
fn parse_time() {
    let d = match dur_from_str(TIME) {
        Ok(d) => {
            assert_eq!(Duration::seconds(3723).add(Duration::nanoseconds(456000000)), d);
            d
        }
        Err(e) => panic!(e),
    };
    let t = Times::from(&d) + Times::from(&d);
    assert!(t.start == d.mul(2) && t.end == d.mul(2));
    assert_eq!("02:04:06,912 --> 02:04:06,912", format!("{}",t))
}

#[test]
fn parse_block() {
    let mut r = BlockReader::new(Cursor::new(BLOCK));
    let b = r.next();
    match b {
        Some(b) => match b {
            Ok(b) => assert_eq!("00:00:22,280 --> 00:00:34,090\nNORIKO\'S DINNER TABLE\n\n", format!("{}", b)),
            Err(e) => panic!(e),
        },
        None => panic!("didn't got any block"),
    }
}
