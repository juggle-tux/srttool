/*
Copyright 2015 juggle-tux

This file is part of srttool.

srttool is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

srttool is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with srttool.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::cmp::Eq;
use std::convert::From;
use std::error::Error;
use std::fmt::{self, Display};
use std::io::{Lines, BufRead};
use std::num::ParseIntError;
use std::ops::{Add, Sub};
use std::time::Duration;

/// start and end time of a subtitle block
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Times {
    pub start: Duration,
    pub end: Duration,
}

impl Times {
    #[inline]
    pub fn new() -> Times {
        Times{start: Duration::new(0, 0), end: Duration::new(0, 0)}
    }
}

impl<'a> From<&'a Duration> for Times {
    #[inline]
    fn from(d: &'a Duration) -> Times {
        Times{start: *d, end: *d}
    }
}

impl Display for Times {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fn h_m_s_ms(d: Duration) -> (u64, u64, u64, u32) {
            let ms = d.subsec_nanos() / 1_000_000;
            let mut t = d.as_secs();
            let s = t % 60;
            t /= 60;
            let m = t % 60;
            let h = t / 60;
            return (h, m, s, ms)
        }
        let (sh, sm, ss, sms) = h_m_s_ms(self.start); // start time
        let (eh, em ,es, ems) = h_m_s_ms(self.end); // end time
        write!(f, "{:0>2}:{:0>2}:{:0>2},{:0>3} --> {:0>2}:{:0>2}:{:0>2},{:0>3}",
               sh, sm, ss, sms, eh, em, es, ems)
            
    }
}

impl Add for Times {
    type Output = Times;

    #[inline]
    fn add(self, rhs: Times) -> Times {
        Times{
            start: self.start.add(rhs.start),
            end: self.end.add(rhs.end),
        }
    }
}

impl Sub for Times {
    type Output = Times;

    #[inline]
    fn sub(self, rhs: Times) -> Times {
        Times{
            start: if self.start.gt(&rhs.start) {
                self.start.sub(rhs.start)
            } else {
                Duration::new(0,0)
            },
            end: if self.end.gt(&rhs.end) {
                self.end.sub(rhs.end)
            } else {
                Duration::new(0,0)
            }
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
    #[inline]
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
            if idx == "" {
                return None //File ends with final new line
            } else if !is_idx(&idx) {
                return Some(Err(ParseError::InvalidIndex));
            }
        } else {
            return None // File ends without final newline
        }

        let time =
            if let Some(Ok(tl)) = self.buf.next() {
                self.line += 1;
                match parse_time_line(&tl) {
                    Ok(time) => time,
                    Err(e) => return Some(Err(e)),
                }
            } else {
                return Some(Err(ParseError::InvalidTimeString))
            };

        let mut content = String::new();
        while let Some(text) = self.buf.next() {
            self.line += 1;
            match text {
                Ok(text) => {
                    if text == "" { break }
                    content = content + &text + "\n";
                }
                Err(_) => {
                    return Some(Err(ParseError::InvalidContent));
                }
            }
        }
        return Some(Ok(Block{times: time, content: content}))
    }
}

#[inline]
fn is_idx(s: &str) -> bool {
    match s.parse::<u64>() {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn parse_time_line(s: &str) -> Result<Times, ParseError> {
    let ts: Vec<&str> = s.splitn(2, " --> ").collect();
    if ts.len() != 2 {
        return Err(ParseError::InvalidTimeLine);
    }
    Ok(Times{
        start: try!(dur_from_str(ts[0])),
        end: try!(dur_from_str(ts[1])),
    })
}

#[derive(Debug)]
pub enum ParseError {
    InvalidTimeString,
    InvalidTimeLine,
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
        self.description().fmt(f)
    }
}

impl Error for ParseError {
    #[inline]
    fn description(&self) -> &'static str {
        match *self {
            ParseError::InvalidIndex => "Invalid index",
            ParseError::InvalidTimeString => "Invalid time",
            ParseError::InvalidContent => "Invalid content",
            ParseError::InvalidTimeLine => "Invalid time line",
        }
    }
}

/// Parse a &str with the format "HH:MM:SS:sss" to a Duration
pub fn dur_from_str(s: &str) -> Result<Duration, ParseError> {
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
    return Ok(dur(h, m, s, ms))
}

#[inline]
fn dur(h: u64, m: u64, s: u64, ms: u32) -> Duration {
    Duration::new(h * 60 * 60
                  +m * 60
                  +s,
                  ms * 1_000_000)
}
