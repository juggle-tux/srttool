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

mod time;
pub use time::{Time, Times};

mod error;
pub use error::ParseError;

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
                match Times::from_str(&tl) {
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
