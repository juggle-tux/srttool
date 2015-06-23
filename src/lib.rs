#![feature(std_misc)]


use std::cmp::Ordering;
use std::convert::From;
use std::error::Error;
use std::fmt::{self, Display};
use std::io::{Lines, BufRead};
use std::num::ParseIntError;
use std::ops::{Add, Neg};
use std::time::Duration;

/// start and end time of a subtitle block
#[derive(Debug, Clone, Copy)]
pub struct Times {
    pub start: Duration,
    pub end: Duration,
}

impl Times {
    pub fn new() -> Times {
        Times{start: Duration::zero(), end: Duration::zero()}
    }
}

impl Eq for Times { }

impl PartialEq for Times {
    fn eq(&self, other: &Times) -> bool {
        self.start.eq(&other.start) && self.end.eq(&other.end)
    }
}

impl Ord for Times {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for Times {
    fn partial_cmp(&self, other: &Times) -> Option<Ordering> {
        self.start.partial_cmp(&other.start)
    }
}



impl<'a> From<&'a Duration> for Times {
    fn from(d: &'a Duration) -> Times {
        Times{start: *d, end: *d}
    }
}

impl From<Duration> for Times {
    fn from(d: Duration) -> Times {
        Times{start: d, end: d}
    }
}

impl Display for Times {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // start time
        let mut t = self.start.num_milliseconds();
        let sms = t % 1000;
        t /= 1000;
        let ss = t % 60;
        t /= 60;
        let sm = t % 60;
        let sh = t / 60;
        // end time
        let mut t = self.end.num_milliseconds();
        let ems = t % 1000;
        t /= 1000;
        let es = t % 60;
        t /= 60;
        let em = t % 60;
        let eh = t / 60;

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
#[derive(Debug, Clone)]
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
    pub line: u64,
}

impl<B: BufRead> BlockReader<B> {
    #[inline]
    pub fn new(buf: B) -> BlockReader<B> {
        BlockReader{buf: buf.lines(), line: 0}
    }
}

impl<B: BufRead> Iterator for  BlockReader<B> {
    type Item = Result<Block, ParseError>;

    /// next returns the next subtitle Block or None at EOF
    fn next(&mut self) -> Option<Result<Block, ParseError>> {
        // idx
        if let Some(Ok(idx)) = self.buf.next() {
            self.line += 1;
            if idx == "" || idx == "\r" {
                return None //File ends with final new line
            } else if !is_idx(&idx) {
                return Some(Err(ParseError::InvalidIndex));
            }
        } else {
            return None // File ends withoout final newline
        }

        // times
        let t = if let Some(Ok(tl)) = self.buf.next() {
            self.line += 1;
            match parse_time_line(&tl) {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            }
        } else {
            return Some(Err(ParseError::InvalidTimeString))
        };

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
    match s.trim_right_matches("\r").parse::<u64>() {
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
pub enum ParseError {
    InvalidTimeString,
    InvalidIndex,
    InvalidContent,
}

impl From<ParseIntError> for ParseError {
    fn from(_: ParseIntError) -> ParseError {
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
    let (neg, s) = if ds.starts_with("n") {
        (true, ds.trim_left_matches("n"))
    } else {
        (false,ds)
    };
    // Vec [hh, mm, ss+ms]
    let tv: Vec<&str> = s.splitn(3, ":").collect();
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
