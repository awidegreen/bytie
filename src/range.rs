use bytesize::ByteSize;
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

use crate::utils::parse_pos;

pub static RANGE_HELP: &str = "RANGE:
  Specify a position/range where the deletion shall be performed on.

  <begin>         Begin to the end of input
  <begin>:<end>   Begin to end (exclusive), requires <end> > <begin>
                  Example: 'foobar', 0:2 == 'fo' or 3:5 == 'ba'
  <begin>:=<end>  Begin to end (inclusive), requires <end> > <begin>
                  Example: 'foobar', 0:=2 == 'foo' or 3:=5 == 'bar'
  <begin>+<count> Begin plus <count> (exclusive), requires <count> > 0.
                  The length includes the begin position: 0+10 is 10 bytes, from 0..9 (same as 0:9)
  <begin>+=<count> Begin plus <count> (inclusive), requires <count> > 0.
                  The length includes the begin position: 0+=10 is 11 bytes, from 0..10 (same as 0:9)
";

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Range {
    pub begin: ByteSize,
    pub end: Option<ByteSize>,
}

impl FromStr for Range {
    type Err = String;

    fn from_str(position: &str) -> std::result::Result<Range, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"^(?P<begin>\d+[kmgtpeibKMGTPEIB]{0,3})(?P<end>(?P<behavior>\+|:|(:=))(?P<to>\d+[kmgtpeibKMGTPEIB]{0,3}))?$"
            )
            .unwrap();
        }

        if let Some(caps) = RE.captures(position) {
            let begin_bytesize = parse_pos(caps.name("begin").unwrap().as_str())?;
            let begin = begin_bytesize.as_u64();
            if caps.name("end").is_some() {
                let to_bytesize = parse_pos(caps.name("to").unwrap().as_str())?;
                let to = to_bytesize.as_u64();

                let end = match caps.name("behavior").unwrap().as_str() {
                    "+" => {
                        if to < 1 {
                            return Err("<count> in POSTION parameter has to be >= 1".into());
                        }
                        begin + to - 1
                    }
                    "+=" => {
                        if to < 1 {
                            return Err("<count> in POSTION parameter has to be >= 1".into());
                        }
                        begin + to
                    }
                    ":" => {
                        if begin >= to {
                            return Err(format!("<end> ({}) in POSITION parameter has to be greater then <begin> ({})", to, begin));
                        }
                        to - 1
                    }
                    ":=" => {
                        if begin >= to {
                            return Err(format!("<end> ({}) in POSITION parameter has to be greater then <begin> ({})", to, begin));
                        }
                        to
                    }
                    x => return Err(format!("Unexpexted position behavior indicator: {}", x)),
                };
                Ok(Range {
                    begin: begin_bytesize,
                    end: Some(ByteSize::b(end)),
                })
            } else {
                Ok(Range {
                    begin: begin_bytesize,
                    end: None,
                })
            }
        } else {
            Err(("Unable to parse <RANGE> parameter").into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_format() {
        let test_vec = vec![
            "a", "", "+", "o", "1000+", "0:", ":", "-1", "+123", "10:10", "3+0", "0+=1", "0:=",
            "0:0", "1:1", "0:-1",
        ];
        for t in test_vec {
            assert!(t.parse::<Range>().is_err());
        }
    }

    #[test]
    fn test_only_begin() {
        let r = "0".parse::<Range>();
        assert!(r.is_ok());
        assert_eq!(
            r.unwrap(),
            Range {
                begin: Default::default(),
                end: None
            }
        );

        let r = "123".parse::<Range>();
        assert!(r.is_ok());
        assert_eq!(
            r.unwrap(),
            Range {
                begin: ByteSize::b(123),
                end: None
            }
        );

        let r = "2kib".parse::<Range>();
        assert!(r.is_ok());
        assert_eq!(
            r.unwrap(),
            Range {
                begin: ByteSize::b(2 * 1024),
                end: None,
            }
        );
    }

    #[test]
    fn test_count() {
        let test_vec = vec![
            ("0+10", ByteSize::b(0), Some(ByteSize::b(9))),
            ("0+1", Default::default(), Some(Default::default())),
        ];

        for (format, begin, end) in test_vec {
            let r = format.parse::<Range>();
            assert!(
                r.is_ok(),
                "Tested: {}, expected begin {} end: {:?}! received err: {}",
                format,
                begin,
                end,
                r.unwrap_err()
            );
            assert_eq!(r.unwrap(), Range { begin, end });
        }
    }

    #[test]
    fn test_range() {
        let test_vec = vec![
            ("0:10", Default::default(), Some(ByteSize::b(9))),
            ("0:=10", Default::default(), Some(ByteSize::b(10))),
            ("512:1024", ByteSize::b(512), Some(ByteSize::b(1023))),
            ("512:=1024", ByteSize::b(512), Some(ByteSize::b(1024))),
            /*
                "3M:=5mib",
                (Bytes::new(3, Unit::MByte).unwrap().size()),
                Some(Bytes::new(5, Unit::MiByte).unwrap().size()),
            ),
            */
        ];

        for (format, begin, end) in test_vec {
            let r = format.parse::<Range>();
            assert!(
                r.is_ok(),
                "Tested: {}, expected begin {} end: {:?}! received err: {}",
                format,
                begin,
                end,
                r.unwrap_err()
            );
            assert_eq!(r.unwrap(), Range { begin, end });
        }
    }
}
