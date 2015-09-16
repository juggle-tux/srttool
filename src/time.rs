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
use std::ops::{Add, Sub};
use std::str::FromStr;
use std::time::Duration;

use error::ParseError;

/// start and end time of a Block
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Times {
    pub start: Time,
    pub end: Time,
}

impl Times {
    #[inline]
    pub fn new() -> Times {
        Times{start: Time::new(), end: Time::new()}
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
            start: self.start.sub(rhs.start),
            end: self.end.sub(rhs.end),
        }
    }
}

impl From<Duration> for Times {
    #[inline]
    fn from(d: Duration) -> Times {
        Times{start: Time::from(d), end: Time::from(d)}
    }
}

impl From<Time> for Times {
    #[inline]
    fn from(t: Time) -> Times {
        Times{start: t, end: t}
    }
}

impl FromStr for Times {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Times, ParseError> {
        let buf: Vec<Time> = s.splitn(2, " --> ")
            .filter_map(|s| Time::from_str(s).ok())
            .collect();

        if buf.len() != 2 {
            return Err(ParseError::InvalidTimeLine);
        }

        return Ok(Times{
            start: buf[0],
            end: buf[1],
        });
    }
}

impl Display for Times {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{} --> {}", self.start, self.end)
    }
}

/// start or end time of a Block
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time(Duration);

impl Time {
    #[inline]
    fn new() -> Time {
        Time(Duration::new(0, 0))
    }
}

impl Add for Time {
    type Output = Time;
    #[inline]
    fn add(self, rhs: Time) -> Time {
        Time(self.0.add(rhs.0))
    }
}

impl Sub for Time {
    type Output = Time;
    #[inline]
    fn sub(self, rhs: Time) -> Time {
        if self.gt(&rhs) {
            Time(self.0.sub(rhs.0))
        } else {
            Time(Duration::new(0,0))
        }
    }
}

impl From<Duration> for Time {
    #[inline]
    fn from(d: Duration) -> Time {
        Time(d)
    }
}

impl From<Time> for Duration {
    #[inline]
    fn from(t: Time) -> Duration {
        t.0
    }
}

impl From<(usize, usize, usize, usize)> for Time {
    #[inline]
    fn from(h_m_s_ms: (usize, usize, usize, usize)) -> Time {
        let (h, m ,s ,ms) = h_m_s_ms;
        Time(Duration::new((h * 60 * 60 + m * 60 +s) as u64,
                           ms as u32 * 1_000_000))
    }
}

/// parses a &str to a Time where &str is "HH:MM:SS,ms"
impl FromStr for Time {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Time, ParseError> {
        let buf: Vec<usize> = s.splitn(2, ",")
            .flat_map(|s| s.splitn(3, ":"))
            .filter_map(|s| s.parse::<usize>().ok())
            .collect();

        if buf.len() != 4 {
            return Err(ParseError::InvalidTimeString);
        }

        return Ok(Time::from((buf[0], buf[1], buf[2], buf[3])))
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let ms = self.0.subsec_nanos() / 1_000_000;
        let mut t = self.0.as_secs();
        let s = t % 60;
        t /= 60;
        let m = t % 60;
        let h = t / 60;
        write!(f, "{:0>2}:{:0>2}:{:0>2},{:0>3}", h, m, s, ms)
    }
}