extern crate clap;

use std::convert::From;
use std::error::Error;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::num;
use std::ops::{Add, Neg};
use std::time::duration::Duration;

pub type Lines = io::Lines<BufReader<File>>;

#[derive(Debug, Clone)]
pub struct Times {
    start: Duration,
    end: Duration,
}

impl Add for Times {
    type Output = Times;

    fn add(self, rhs: Times) -> Times {
        Times{
            start: self.start.add(rhs.start),
            end: self.end.add(rhs.end),
        }
    }
}

/// single subtitle block
#[derive(Debug, Clone)]
pub struct Block {
    times: Times,
    content: String,
}

pub struct BlockReader {
    l: Lines,
}

impl BlockReader {
    //#[inline]
    pub fn new(l: Lines) -> BlockReader {
        BlockReader{l: l}
    }
}

impl Iterator for BlockReader {
    type Item = Block;

    fn next(&mut self) -> Option<Block> {
        // idx
        if let Some(Ok(idx)) = self.l.next() {
            println!("{:?}", idx);
            if !is_idx(&idx) { return None }
        }

        // time
        let t: Times;
        match self.l.next() {
            Some(Ok(tl)) => {
                match parse_time_line(&tl) {
                    Ok(tt) => t=tt,
                    Err(_) => return None,
                }
            }
            _ => return None,
        }

        // content
        let mut c: String = "".to_string();
        while let Some(Ok(text)) = self.l.next() {
            if text == "\r" { break }
            c = c + &text.trim_right_matches("\r") + "\n";
        } 
        return Some(Block{
            times: t,
            content: c,
        })
    }
}
      
fn is_idx(s: &str) -> bool {
    match s.trim_right_matches("\r").parse::<i64>() {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn parse_time_line(s: &str) -> Result<Times, ParseError> {
    let ts: Vec<&str> = s.splitn(2, " --> ").collect();
    if ts.len() != 2 {
        return Err(ParseError::InvalidTimeString);
    }
    if let Ok(st) = dur_from_str(ts[0].trim_right_matches("\r")) {
        if let Ok(et) = dur_from_str(ts[1].trim_right_matches("\r")) {
            return Ok(Times{start: st, end: et})
        }
    }
    return Err(ParseError::InvalidTimeString)
}

#[derive(Debug)]
pub enum ParseError { InvalidTimeString }

impl From<num::ParseIntError> for ParseError {
    fn from(_: num::ParseIntError) -> ParseError {
        return ParseError::InvalidTimeString
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}

impl Error for ParseError {
    fn description(&self) -> &'static str {
        return "Invalid time"
    }
}

// Parse a &str with the format "HH:MM:SS:sss" to a Duration
pub fn dur_from_str(ds: &str) -> Result<Duration, ParseError> {
    let neg;
    let s;
    if ds.starts_with("n") {
        neg = true;
        s = ds.trim_left_matches("n");
    } else {
        neg = false;
        s = ds;
    }
    // Vec [hh, mm, ss+ms]
    let tv: Vec<_> = s.splitn(3, ":").collect();
    if tv.len() != 3 {
        return Err(ParseError::InvalidTimeString);
    }
    // Vec [ss, ms]
    let sv: Vec<_> = tv[2].splitn(2, ",").collect();
    if sv.len() != 2 {
        return Err(ParseError::InvalidTimeString);
    }

    let h = try!(tv[0].parse());
    let m = try!(tv[1].parse());
    let s = try!(sv[0].parse());
    let ms = try!(sv[1].parse());
    if neg {
        return Ok(dur(h, m, s, ms).neg())
    }
    return Ok(dur(h, m, s, ms))
}

#[inline]
fn dur(h: i64, m: i64, s: i64, ms: i64) -> Duration {
    Duration::milliseconds(h * 60 * 60 * 1000
                           +m * 60 * 1000
                           +s * 1000
                           +ms)
}
