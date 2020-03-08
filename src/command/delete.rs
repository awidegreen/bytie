use crate::position::Position;
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
        let position = self.position.parse::<Position>()?;

        let mut buffer = vec![0; blocksize];
        let mut total_read = 0;
        let mut end = 0;
        let mut del_to_end = false;

        if let Some(pend) = position.end {
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
                        let offset = position.begin - (total_read - n);
                        out.write(&buffer[0..offset])?;

                        if del_to_end {
                            break;
                        } else if end < total_read {
                            //let offset_end = (end % n) + 1;
                            let offset_end = (end - (total_read - n)) + 1;
                            out.write(&buffer[offset_end..n])?;
                            State::Done
                        } else {
                            State::Skip
                        }
                    } else {
                        out.write(&buffer[0..n])?;
                        State::Write
                    }
                }
                State::Skip => {
                    if total_read > end {
                        let offset = (end - (total_read - n)) + 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;

    #[test]
    fn test_small_blocksize() {
        let mut cmd = DeleteCommand {
            position: "".to_string(),
        };
        let input = "HelloWelt!";
        let mut out: Vec<u8> = vec![];

        for bs in vec![1, 2, 3, 4, 10] {
            for start in 0..=input.len() {
                for end in start + 1..input.len() {
                    let mut exp = String::from(&input[0..start]);
                    exp = exp + &input[end + 1..];
                    out.clear();
                    cmd.position = format!("{}:={}", start, end);
                    let r = cmd.run(bs, &mut input.as_bytes(), &mut out, None);
                    assert!(r.is_ok(), "Error: {:?}", r.unwrap_err());
                    let out = std::str::from_utf8(&out).unwrap();
                    assert_eq!(exp, out);
                }
            }
        }
    }

    #[test]
    fn test_big_blocksize() {
        let mut cmd = DeleteCommand {
            position: "".to_string(),
        };
        let input = r##"Lorem ipsum dolor sit amet, consectetur adipiscing elit,
            sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
            Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris
            nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in
            reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla
            pariatur. Excepteur sint occaecat cupidatat non proident, sunt in
            culpa qui officia deserunt mollit anim id est laborum"##;
        let mut out: Vec<u8> = vec![];

        for bs in vec![32, 64, 128, 512, 1024, 2048] {
            for start in 0..=input.len() {
                for end in start + 1..input.len() {
                    let mut exp = String::from(&input[0..start]);
                    exp = exp + &input[end + 1..];
                    out.clear();
                    cmd.position = format!("{}:={}", start, end);
                    assert!(cmd.run(bs, &mut input.as_bytes(), &mut out, None).is_ok());
                    let out = std::str::from_utf8(&out).unwrap();
                    assert_eq!(exp, out);
                }
            }
        }
    }
}
