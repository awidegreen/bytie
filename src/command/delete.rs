use clap::{value_t, ArgMatches};
use failure::{bail, Error};
use log::debug;

pub struct DeleteCommand {
    position: String,
}
impl DeleteCommand {
    pub fn from_matches(m: &ArgMatches) -> Result<Self, Error> {
        let position = value_t!(m, "position", String)?;
        Ok(Self { position })
    }
}

impl crate::command::Command for DeleteCommand {
    fn run(
        &self,
        blocksize: usize,
        source: &mut dyn std::io::Read,
        out: &mut dyn std::io::Write,
        _input: Option<&mut dyn std::io::Read>,
    ) -> Result<(), Error> {
        let position = crate::utils::parse_position(&self.position)?;

        let mut buffer = vec![0; blocksize];
        let mut total_read = 0;
        let mut end = 0;
        let mut del_to_end = false;

        if let Some(pend) = position.end {
            if pend < 0 {
                bail!("Unable to handle negative end")
            }
            end = pend as usize;
            if end < position.begin {
                bail!("End must be greater than begin")
            }
        } else {
            del_to_end = true;
        }
        //debug!(
        //"deltoend: {} begin: {} end: {}",
        //del_to_end, position.begin, end
        //);

        #[derive(Debug)]
        enum State {
            Write,
            Skip,
            Done,
        };
        let mut state = State::Write;

        loop {
            let n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            total_read = total_read + n;

            debug!(
                "n: {} total_read: {} begin: {} end: {}: state: {:?}",
                n, total_read, position.begin, end, state
            );
            state = match state {
                State::Write => {
                    if total_read > position.begin {
                        let offset = position.begin % n;
                        //debug!("offset: {}", offset);
                        //debug!("write cond: {:?}", &buffer[0..offset]);
                        out.write(&buffer[0..offset])?;

                        if del_to_end {
                            break;
                        } else if end < total_read {
                            let offset_end = (end % n) + 1;
                            //debug!("offset_end: {}", offset_end);
                            //debug!("write cond w/ end: {:?}", &buffer[offset_end..n]);
                            out.write(&buffer[offset_end..n])?;
                            State::Done
                        } else {
                            State::Skip
                        }
                    } else {
                        //debug!("write else: {:?}", &buffer[0..n]);
                        out.write(&buffer[0..n])?;
                        State::Write
                    }
                }
                State::Skip => {
                    if total_read > end {
                        let offset = (end % n) + 1;
                        //debug!("offset: {}", offset);
                        out.write(&buffer[offset..n])?;
                        State::Done
                    } else {
                        State::Skip
                    }
                }
                State::Done => {
                    out.write(&buffer[0..n])?;
                    State::Done
                }
            };
        }
        out.flush()?;
        Ok(())
    }
}

//#[cfg(test)]
//mod tests {
//use super::*;
//use crate::command::Command;

//#[test]
//fn test_iterate_over_all() {
//let mut cmd = DeleteCommand {
//position: "".to_string(),
//};
//let input = "foobar\n";
//let mut out: Vec<u8> = vec![];

//for bs in vec![1, 2, 3, 4, 10, 32, 64] {
//for start in 1..4 {
//for end in start..input.len() {
//let exp = &input[start..end + 1];
//out.clear();
//cmd.position = format!("{}:{}", start, end);
//assert!(cmd.run(bs, &mut input.as_bytes(), &mut out, None).is_ok());
//let out = std::str::from_utf8(&out).unwrap();
//assert_eq!(exp, out);
//}
//}
//}
//}
//}
