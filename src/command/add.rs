use clap::{value_t, ArgMatches};
use failure::{bail, Error};

pub struct AddCommand {
    begin: i64,
    value: Option<String>,
}
impl AddCommand {
    pub fn from_matches(m: &ArgMatches) -> Result<Self, Error> {
        let begin = value_t!(m, "begin", i64)?;
        let value = value_t!(m, "value", String).ok();

        Ok(Self { begin, value })
    }
}

impl crate::command::Command for AddCommand {
    fn run(
        &self,
        blocksize: usize,
        source: &mut dyn std::io::Read,
        out: &mut dyn std::io::Write,
        input: Option<&mut dyn std::io::Read>,
    ) -> Result<(), Error> {
        let begin = if self.begin == -1 {
            std::usize::MAX
        } else {
            self.begin as usize
        };

        if self.value.is_none() && input.is_none() {
            bail!("Well, as no <VALUE> input parameter has been provided, some input should be provided by STDIN.")
        }

        let mut buffer = vec![0; blocksize];
        let mut total_read = 0;
        let mut offset = 0;
        let mut n;

        loop {
            n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            total_read = total_read + n;
            if total_read > begin {
                offset = begin % n;
                out.write(&buffer[0..offset])?;
                break;
            } else {
                out.write(&buffer[0..n])?;
            }
        }

        if let Some(input) = input {
            let mut buffer = vec![0; blocksize];
            loop {
                let n = input.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                out.write(&buffer[0..n])?;
            }
        } else {
            if let Some(ref value) = self.value {
                out.write(value.as_ref())?;
            } else {
                bail!("No stdin nor any value has been provided")
            }
        }

        if offset <= n {
            out.write(&buffer[offset..n])?;
        }

        while n != 0 {
            n = source.read(&mut buffer)?;
            out.write(&buffer[0..n])?;
        }

        out.flush()?;
        Ok(())
    }
}
