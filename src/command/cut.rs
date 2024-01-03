use crate::range::Range;
use anyhow::{anyhow, Result};
use clap::Args;
use log::debug;

#[derive(Args, Debug)]
#[command(name = "cut", visible_alias = "extract")]
pub(crate) struct CutCommand {
    /// Specifies a range/count for the operation, see RANGE section
    pub range: Range,
}

impl crate::command::Command for CutCommand {
    fn run(
        &self,
        blocksize: usize,
        source: &mut dyn std::io::Read,
        out: &mut dyn std::io::Write,
        _input: Option<&mut dyn std::io::Read>,
    ) -> Result<()> {
        let mut buffer = vec![0; blocksize];
        let mut total_read = 0;
        let mut end = 0;
        let mut cut_till_end = false;

        let begin = self.range.begin.as_u64() as usize;

        if let Some(pend) = self.range.end {
            end = pend.as_u64() as usize;
            if end < begin {
                return Err(anyhow!("End must be greater than begin"));
            }
        } else {
            cut_till_end = true;
        }

        #[derive(Debug)]
        enum State {
            Write, // print to out
            Skip,  // don't print to out
        }
        let mut state = State::Skip;

        loop {
            let n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            total_read += n;

            debug!(
                "n: {} total_read: {}, begin: {}, end: {}, state: {:?}",
                n, total_read, begin, end, state
            );
            state = match state {
                State::Write => {
                    if cut_till_end {
                        out.write_all(&buffer[0..n])?;
                        State::Write
                    } else if total_read > end {
                        let offset_end = (end - (total_read - n)) + 1;
                        out.write_all(&buffer[0..offset_end])?;
                        break; // no need to read more
                    } else {
                        out.write_all(&buffer[0..n])?;
                        State::Write
                    }
                }
                State::Skip => {
                    if total_read > begin {
                        let offset = begin - (total_read - n);
                        if total_read > end && !cut_till_end {
                            let offset_end = (end - (total_read - n)) + 1;
                            out.write_all(&buffer[offset..offset_end])?;
                            break;
                        } else {
                            out.write_all(&buffer[offset..n])?;
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
    fn test_small_blocksize() {
        let input = "HalloWelt";
        let mut out: Vec<u8> = vec![];

        for bs in vec![1, 2, 3, 4, 10] {
            for start in 0..=input.len() {
                for end in start + 1..input.len() {
                    let exp = &input[start..end + 1];
                    out.clear();
                    let cmd = CutCommand {
                        range: format!("{}:={}", start, end).parse().unwrap(),
                    };
                    assert!(cmd.run(bs, &mut input.as_bytes(), &mut out, None).is_ok());
                    let out = std::str::from_utf8(&out).unwrap();
                    assert_eq!(exp, out);
                }
            }
        }
    }

    #[test]
    fn test_big_blocksize() {
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
                    let exp = &input[start..end + 1];
                    out.clear();
                    let cmd = CutCommand {
                        range: format!("{}:={}", start, end).parse().unwrap(),
                    };
                    assert!(cmd.run(bs, &mut input.as_bytes(), &mut out, None).is_ok());
                    let out = std::str::from_utf8(&out).unwrap();
                    assert_eq!(exp, out, "bs: {}, start: {}, end: {}", bs, start, end);
                }
            }
        }
    }
}
