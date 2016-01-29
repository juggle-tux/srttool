// Copyright 2015 juggle-tux
//
// This file is part of srttool.
//
// srttool is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// srttool is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with srttool.  If not, see <http://www.gnu.org/licenses/>.
//

use std::convert::From;
use std::error::Error;
use std::fmt::{self, Display};
use std::num::ParseIntError;

/// Parsing errors
#[derive(Debug, Copy, Clone)]
pub enum ParseError {
    /// The time is not formatted properly
    InvalidTimeString,

    /// The line with the times is not formatted properly
    InvalidTimeLine,

    /// The index ist not a integer
    InvalidIndex,

    /// A error while reading the content
    InvalidContent,
}

impl From<ParseIntError> for ParseError {
    fn from(_: ParseIntError) -> ParseError {
        return ParseError::InvalidTimeString;
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}

impl Error for ParseError {
    fn description(&self) -> &'static str {
        match *self {
            ParseError::InvalidIndex => "Invalid index",
            ParseError::InvalidTimeString => "Invalid time",
            ParseError::InvalidContent => "Invalid content",
            ParseError::InvalidTimeLine => "Invalid time line",
        }
    }
}
