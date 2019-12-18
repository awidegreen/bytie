use clap::{value_t, ArgMatches};
use failure::{bail, Error};
use log::debug;

pub struct CutCommand {
    position: String,
}

impl CutCommand {
    pub fn from_matches(m: &ArgMatches) -> Result<Self, Error> {
        let position = value_t!(m, "position", String)?;
        Ok(Self { position })
    }
}

impl crate::command::Command for CutCommand {
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
        let mut cut_till_end = false;

        if let Some(pend) = position.end {
            if pend < 0 {
                bail!("Unable to handle negative end")
            }
            end = pend as usize;
            if end < position.begin {
                bail!("End must be greater than begin")
            }
        } else {
            cut_till_end = true;
        }

        #[derive(Debug)]
        enum State {
            Write, // print to out
            Skip,  // don't print to out
        };
        let mut state = State::Skip;

        loop {
            let n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            total_read = total_read + n;

            println!(
                "n: {} total_read: {}, begin: {}, end: {}, state: {:?}",
                n, total_read, position.begin, end, state
            );
            state = match state {
                State::Write => {
                    if cut_till_end {
                        out.write(&buffer[0..n])?;
                        State::Write
                    } else {
                        if total_read > end {
                            let offset = (end % n) + 1;
                            println!("In Write, n: {}, offset: {}", n, offset);
                            out.write(&buffer[0..offset])?;
                            break; // no need to read more
                        } else {
                            out.write(&buffer[0..n])?;
                            State::Write
                        }
                    }
                }
                State::Skip => {
                    if total_read > position.begin {
                        let offset = position.begin % n;
                        println!("In skip, offset: {}", offset);
                        if total_read > end && !cut_till_end {
                            let offset_end = (end % n) + 1;
                            out.write(&buffer[offset..offset_end])?;
                            break;
                        } else {
                            out.write(&buffer[offset..n])?;
                            State::Write
                        }
                    } else {
                        State::Skip
                    }
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
    fn test_iterate_over_all() {
        let mut cmd = CutCommand {
            position: "".to_string(),
        };
        let input = "foobar\n";
        let mut out: Vec<u8> = vec![];

        for bs in vec![1, 2, 3, 4, 10, 32, 64] {
            for start in 1..4 {
                for end in start..6 {
                    println!(
                        "Run with bs: {}, start: {} end: {}, exp: {}",
                        bs,
                        start,
                        end,
                        &input[start..end + 1]
                    );
                    out.clear();
                    cmd.position = format!("{}:{}", start, end);
                    assert!(cmd.run(bs, &mut input.as_bytes(), &mut out, None).is_ok());
                    let out = std::str::from_utf8(&out).unwrap();
                    assert_eq!(&input[start..end + 1], out);
                }
            }
        }
    }
}
