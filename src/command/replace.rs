use clap::{value_t, ArgMatches};
use failure::{bail, Error};

pub struct ReplaceCommand {
    begin: usize,
    value: Option<String>,
}
impl ReplaceCommand {
    pub fn from_matches(m: &ArgMatches) -> Result<Self, Error> {
        let begin = value_t!(m, "begin", usize)?;
        let value = value_t!(m, "value", String).ok();

        Ok(Self { begin, value })
    }
}

impl crate::command::Command for ReplaceCommand {
    fn run(
        &self,
        blocksize: usize,
        source: &mut dyn std::io::Read,
        out: &mut dyn std::io::Write,
        input: Option<&mut dyn std::io::Read>,
    ) -> Result<(), Error> {
        if self.value.is_none() && input.is_none() {
            bail!("Well, as no <VALUE> input parameter has been provided, some input should be provided by STDIN.")
        }

        let mut buffer = vec![0; blocksize];
        let mut in_total_read = 0;
        let mut offset = 0;
        let mut n;
        let mut total_written = 0;

        loop {
            n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            in_total_read = in_total_read + n;
            if in_total_read > self.begin {
                offset = self.begin % n;
                out.write(&buffer[0..offset])?;
                total_written = total_written + offset;
                break;
            } else {
                out.write(&buffer[0..n])?;
                total_written = total_written + n;
            }
        }

        let written = if let Some(input) = input {
            let mut written = 0;
            let mut b = vec![0; blocksize];
            loop {
                let n = input.read(&mut b)?;
                if n == 0 {
                    break;
                }
                out.write(&b[0..n])?;
                written = written + n;
            }
            written
        } else {
            if let Some(ref value) = self.value {
                out.write(value.as_ref())?;
                value.len()
            } else {
                bail!("No stdin nor any value has been provided")
            }
        };

        if offset <= n && written < (n - offset) {
            offset = offset + written;
            out.write(&buffer[offset..n])?;
            total_written = total_written + written;
        }

        if total_written > in_total_read {
            loop {
                n = source.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                in_total_read = in_total_read + n;
                if in_total_read > total_written {
                    offset = total_written % n;
                    out.write(&buffer[offset..n])?;
                    break;
                } else if in_total_read == total_written {
                    break;
                }
            }
        }
        loop {
            let n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            out.write(&buffer[0..n])?;
        }

        out.flush()?;
        Ok(())
    }
}
