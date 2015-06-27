#![feature(std_misc)] #![feature(std_misc)]
extern crate srt;

//use std::io::prelude::*;
use std::io::Cursor;
use srt::*;
use std::time::Duration;
use std::ops::{Add, Mul, Neg};

const BLOCK: &'static [u8] = b"1\n00:00:22,280 --> 00:00:34,090\nNORIKO'S DINNER TABLE\n\n";

#[test]
fn parse_time() {
    let d = match dur_from_str("01:02:03,456") {
        Ok(d) => {
            assert_eq!(Duration::seconds(3723).add(Duration::nanoseconds(456000000)), d);
            d
        }
        Err(e) => panic!(e),
    };
    let mut t = Times::from(&d);
    t.end = t.end.add(dur_from_str("6:5:4,321").unwrap());
    assert_eq!(format!("{}",t), "01:02:03,456 --> 07:07:07,777");

    t = Times::from(&Duration::seconds(1).neg());
    assert_eq!(format!("{}", t), "00:00:00,000 --> 00:00:00,000");

    t = Times::from(&dur_from_str("99:99:99,999").unwrap());
    t.end = t.end.mul(2);
    assert_eq!(format!("{}", t), "100:40:39,999 --> 201:21:19,998");
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
