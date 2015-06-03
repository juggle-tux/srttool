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

#[derive(Debug, Clone, Copy)]
pub struct Times {
    start: Duration,
    end: Duration,
}

impl<'a> From<&'a Duration> for Times {
    fn from(d: &'a Duration) -> Times {
        Times{start: *d, end: *d}
    }
}

impl Display for Times {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // start time
        let sh = self.start.num_hours();
        let o = sh * 60;
        let sm = self.start.num_minutes() - o;
        let o = o * 60 + sm * 60;
        let ss = self.start.num_seconds() - o;
        let o = o * 1000 + ss * 1000;
        let sms = self.start.num_milliseconds() - o;

        // end time
        let eh = self.end.num_hours();
        let o = eh * 60;
        let em = self.end.num_minutes() - o;
        let o = o * 60 + em * 60;
        let es = self.end.num_seconds() - o;
        let o = o * 1000 + es * 1000;
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

pub struct BlockReader {
    l: Lines,
    err: Option<ParseError>,
    pub line: i64,
}

impl BlockReader {
    //#[inline]
    pub fn new(l: Lines) -> BlockReader {
        BlockReader{l: l, err: None, line: 0}
    }
    
    pub fn err(self) -> Option<ParseError> {
        self.err
    }

    pub fn line_nr(self) -> i64 {
        self.line
    }
}

impl Iterator for  BlockReader {
    type Item = Block;

    fn next(&mut self) -> Option<Block> {
        // idx
        if let Some(Ok(idx)) = self.l.next() {
            self.line += 1;
            if idx == "" || idx == "\r" { return None }
            else if !is_idx(&idx) {
                self.err = Some(ParseError::InvalidIndex);
                return None
            }
        } else { return None }

        // time
        let t: Times;
        match self.l.next() {
            Some(Ok(tl)) => {
                self.line += 1;
                match parse_time_line(&tl) {
                    Ok(tt) => t=tt,
                    Err(e) => {
                        self.err = Some(e);
                        return None
                    }
                }
            }
            _ => {
                self.err = Some(ParseError::InvalidTimeString);
                return None
            }
        }

        // content
        let mut c: String = "".to_string();
        while let Some(text) = self.l.next() {
            self.line += 1;
            match text {
                Ok(text) => {
                    if text == "\r" || text == "" { break }
                    c = c + &text.trim_right_matches("\r") + "\n";
                }
                Err(_) => {
                    self.err = Some(ParseError::InvalidContent);
                    return None;
                }
            }
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
