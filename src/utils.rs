use failure::{bail, Error};
use regex::Regex;

use lazy_static::lazy_static;

#[derive(Debug, PartialEq, Eq)]
pub struct Position {
    pub begin: usize,
    pub end: Option<i64>,
}

pub fn parse_position(position: &str) -> Result<Position, Error> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^(?P<begin>\d+)(?P<end>(?P<behavior>\+|:)(?P<to>-?\d+))?$").unwrap();
    }

    if let Some(caps) = RE.captures(position) {
        let begin = caps.name("begin").unwrap().as_str();
        let begin = begin.parse::<usize>()?;
        if let Some(_) = caps.name("end") {
            let to = caps.name("to").unwrap().as_str();
            let to = to.parse::<i64>()?;

            let end = match caps.name("behavior").unwrap().as_str() {
                "+" => begin as i64 + to,
                ":" => to,
                x => bail!("Unexpexted position behavior indicator: {}", x),
            };
            Ok(Position {
                begin,
                end: Some(end),
            })
        } else {
            Ok(Position { begin, end: None })
        }
    } else {
        bail!("Unable to parse <POSITION> parameter");
    }
}

mod tests {
    //use super::*;
    //#[test]
    //fn test_only_begin() {
    //parse_position("123");
    //}
}
