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

use std::error::Error;
use std::fmt::{self, Display};
use std::io::{Lines, BufRead};
use std::str::FromStr;
use std::time::Duration;
use std::ops::{Add, Sub};

pub use self::error::ParseError;
pub use self::time::{Time, StartEnd};

mod error;
mod time;

/// single subtitle block
#[derive(Debug, Clone)]
pub struct Block {
    pub start_end: StartEnd,
    pub content: String,
}

impl Add<StartEnd> for Block {
    type Output = Block;
    fn add(self, rhs: StartEnd) -> Block {
        return Block{
            start_end: self.start_end + rhs,
            content: self.content,
        }
    }
}

impl Add<Time> for Block {
    type Output = Block;
    fn add(self, rhs: Time) -> Block {
        return Block{
            start_end: self.start_end + rhs,
            content: self.content,
        }
    }
}

impl Add<Duration> for Block {
    type Output = Block;
    fn add(self, rhs: Duration) -> Block {
        return Block{
            start_end: self.start_end + rhs,
            content: self.content,
        }
    }
}

impl Sub<StartEnd> for Block {
    type Output = Block;
    fn sub(self, rhs: StartEnd) -> Block {
        return Block{
            start_end: self.start_end - rhs,
            content: self.content,
        }
    }
}

impl Sub<Time> for Block {
    type Output = Block;
    fn sub(self, rhs: Time) -> Block {
        return Block{
            start_end: self.start_end - rhs,
            content: self.content,
        }
    }
}

impl Sub<Duration> for Block {
    type Output = Block;
    fn sub(self, rhs: Duration) -> Block {
        return Block{
            start_end: self.start_end - rhs,
            content: self.content,
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}\n{}\n", self.start_end, self.content)
    }
}

/// a BlockReader
pub struct BlockReader<B> {
    buf: Lines<B>,
    pub line: u64,
}

impl<B: BufRead> BlockReader<B> {
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
                match StartEnd::from_str(&tl) {
                    Ok(time) => time,
                    Err(e) => return Some(Err(e)),
                }
            } else {
                return Some(Err(ParseError::InvalidTimeString))
            };

        let mut content = String::with_capacity(128);
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
        return Some(Ok(Block{start_end: time, content: content}))
    }
}

fn is_idx(s: &str) -> bool {
    match s.parse::<u64>() {
        Ok(_) => true,
        Err(_) => false,
    }
}
