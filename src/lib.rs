#![feature(std_misc)]
#![allow(dead_code)]

use std::convert::From;
use std::error::Error;
use std::fmt::{self, Display};
use std::io::prelude::*;
use std::io::{self, Lines};
use std::num;
use std::ops::{Add, Neg};
use std::time::duration::Duration;

/// start and end time of a subtitle block
#[derive(Debug, Clone, Copy)]
pub struct Times {
    start: Duration,
    end: Duration,
}

impl Times {
    pub fn new() -> Times {
        Times{start: Duration::zero(), end: Duration::zero()}
    }
}

impl<'a> From<&'a Duration> for Times {
    fn from(d: &'a Duration) -> Times {
        Times{start: *d, end: *d}
    }
}

impl Display for Times {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // start time
        let mut o;
        let sh = self.start.num_hours();
        o = sh * 60;
        let sm = self.start.num_minutes() - o;
        o = 60 * (o + sm);
        let ss = self.start.num_seconds() - o;
        o = 1000 * (o + ss);
        let sms = self.start.num_milliseconds() - o;

        // end time
        let eh = self.end.num_hours();
        o = eh * 60;
        let em = self.end.num_minutes() - o;
        o = 60 * (o + em);
        let es = self.end.num_seconds() - o;
        o = 1000 * (o + es);
        let ems = self.end.num_milliseconds() - o;

        write!(f, "{:0>2}:{:0>2}:{:0>2},{:0>3} --> {:0>2}:{:0>2}:{:0>2},{:0>3}",
               sh, sm, ss, sms, eh, em, es, ems)
    
    }
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
#[derive(Debug)]
pub struct Block {
    pub times: Times,
    pub content: String,
}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}\n{}\n", self.times, self.content)    
    }
}

/// a BlockReader
pub struct BlockReader<B> {
    buf: Lines<B>,
    line: u64,
}

impl<B: BufRead> BlockReader<B> {
    #[inline]
    pub fn new(buf: B) -> BlockReader<B> {
        BlockReader{buf: buf.lines(), line: 0}
    }

    #[inline]
    pub fn line_nr(&self) -> u64 {
        self.line
    }
}

impl<B: BufRead> Iterator for  BlockReader<B> {
    type Item = Result<Block, ParseError>;

    /// next returns the next subtitle Block or None at EOF
    fn next(&mut self) -> Option<Result<Block, ParseError>> {
        // idx
        if let Some(Ok(idx)) = self.buf.next() {
            self.line += 1;
            if idx == "" || idx == "\r" { return None } // File ends with a final newline
            else if !is_idx(&idx) {
                return Some(Err(ParseError::InvalidIndex));
            }
        } else { return None } // File ends withoout final newline

        // time
        let t: Times;
        if let Some(Ok(tl)) = self.buf.next() {
            self.line += 1;
            match parse_time_line(&tl) {
                Ok(tt) => t=tt,
                Err(e) => {
                    return Some(Err(e))
                }
            }
        } else {
            return Some(Err(ParseError::InvalidTimeString))
        }

        // content
        let mut c = String::new();
        while let Some(text) = self.buf.next() {
            self.line += 1;
            match text {
                Ok(text) => {
                    if text == "\r" || text == "" { break }
                    c = c + &text.trim_right_matches("\r") + "\n";
                }
                Err(_) => {
                    return Some(Err(ParseError::InvalidContent));
                }
            }
        }
        return Some(Ok(Block{times: t, content: c}))
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
pub enum ParseError { InvalidTimeString, InvalidIndex, InvalidContent }

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
        match *self {
            ParseError::InvalidIndex => "Invalid index",
            ParseError::InvalidTimeString => "Invalid time",
            ParseError::InvalidContent => "Invalid content",
        }
    }
}

/// Parse a &str with the format "HH:MM:SS:sss" to a Duration
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
    } else {
        return Ok(dur(h, m, s, ms))
    }
}

#[inline]
fn dur(h: i64, m: i64, s: i64, ms: i64) -> Duration {
    Duration::milliseconds(h * 60 * 60 * 1000
                           +m * 60 * 1000
                           +s * 1000
                           +ms)
}
