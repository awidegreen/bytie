use failure::{bail, Error};
use humanize_rs::bytes::Bytes;
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Position {
    pub(crate) begin: usize,
    pub(crate) end: Option<usize>,
}

impl FromStr for Position {
    type Err = Error;

    fn from_str(position: &str) -> Result<Position, Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"^(?P<begin>\d+[kmgtpeibKMGTPEIB]{0,3})(?P<end>(?P<behavior>\+|:|(:=))(?P<to>\d+[kmgtpeibKMGTPEIB]{0,3}))?$"
            )
            .unwrap();
        }

        if let Some(caps) = RE.captures(position) {
            let begin = caps.name("begin").unwrap().as_str().parse::<Bytes>()?;
            let begin = begin.size();
            if let Some(_) = caps.name("end") {
                let to = caps.name("to").unwrap().as_str().parse::<Bytes>()?;
                let to = to.size();

                let end = match caps.name("behavior").unwrap().as_str() {
                    "+" => {
                        if to < 1 {
                            bail!("<count> in POSTION parameter has to be >= 1")
                        }
                        begin + to - 1
                    }
                    ":" => {
                        if begin >= to {
                            bail!("<end> ({}) in POSITION parameter has to be greater then <begin> ({})", to, begin)
                        }
                        to - 1
                    }
                    ":=" => {
                        if begin >= to {
                            bail!("<end> ({}) in POSITION parameter has to be greater then <begin> ({})", to, begin)
                        }
                        to
                    }
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
}

#[cfg(test)]
mod tests {
    use super::Position;
    use humanize_rs::bytes::{Bytes, Unit};

    #[test]
    fn test_invalid_format() {
        let test_vec = vec![
            "a", "", "+", "o", "1000+", "0:", ":", "-1", "+123", "10:10", "3+0", "0+=1", "0:=",
            "0:0", "1:1", "0:-1",
        ];
        for t in test_vec {
            assert!(t.parse::<Position>().is_err());
        }
    }

    #[test]
    fn test_only_begin() {
        let r = "0".parse::<Position>();
        assert!(r.is_ok());
        assert_eq!(
            r.unwrap(),
            Position {
                begin: 0,
                end: None
            }
        );

        let r = "123".parse::<Position>();
        assert!(r.is_ok());
        assert_eq!(
            r.unwrap(),
            Position {
                begin: 123,
                end: None
            }
        );

        let r = "2kib".parse::<Position>();
        assert!(r.is_ok());
        assert_eq!(
            r.unwrap(),
            Position {
                begin: 2 * 1024,
                end: None
            }
        );
    }

    #[test]
    fn test_count() {
        let test_vec = vec![("0+10", 0, Some(9)), ("0+1", 0, Some(0))];

        for (format, begin, end) in test_vec {
            let r = format.parse::<Position>();
            assert!(
                r.is_ok(),
                "Tested: {}, expected begin {} end: {:?}! received err: {}",
                format,
                begin,
                end,
                r.unwrap_err()
            );
            assert_eq!(r.unwrap(), Position { begin, end });
        }
    }

    #[test]
    fn test_range() {
        let test_vec = vec![
            ("0:10", 0, Some(9)),
            ("0:=10", 0, Some(10)),
            ("512:1024", 512, Some(1023)),
            ("512:=1024", 512, Some(1024)),
            (
                "3M:=5mib",
                (Bytes::new(3, Unit::MByte).unwrap().size()),
                Some(Bytes::new(5, Unit::MiByte).unwrap().size()),
            ),
        ];

        for (format, begin, end) in test_vec {
            let r = format.parse::<Position>();
            assert!(
                r.is_ok(),
                "Tested: {}, expected begin {} end: {:?}! received err: {}",
                format,
                begin,
                end,
                r.unwrap_err()
            );
            assert_eq!(r.unwrap(), Position { begin, end });
        }
    }
}
