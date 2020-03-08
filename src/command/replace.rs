use clap::{value_t, ArgMatches};
use failure::{bail, Error};
use humanize_rs::bytes::Bytes;

pub struct ReplaceCommand {
    begin: usize,
    value: Option<Vec<u8>>,
}
impl ReplaceCommand {
    pub fn from_matches(m: &ArgMatches) -> Result<Self, Error> {
        let begin = value_t!(m, "begin", String)?;
        let begin = begin.parse::<Bytes>()?.size();
        let value = if let Ok(value) = value_t!(m, "value", String) {
            Some(value.as_bytes().to_vec())
        } else {
            None
        };

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

        // read from source until begin position
        loop {
            n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            in_total_read = in_total_read + n;
            if in_total_read > self.begin {
                offset = self.begin - (in_total_read - n);
                out.write(&buffer[0..offset])?;
                total_written = total_written + offset;
                break;
            } else {
                out.write(&buffer[0..n])?;
                total_written = total_written + n;
            }
        }

        // written the input data, and remember how much data has been written
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
                bail!("No STDIN nor any <VALUE> has been provided, unable to proceed.")
            }
        };

        // if input data (replace with) is shorter than the block that has been
        // read before, make sure the current block is finished before reading
        // next block from source.
        if offset <= n && written < (n - offset) {
            offset = offset + written;
            out.write(&buffer[offset..n])?;
        }
        total_written = total_written + written;

        if total_written > in_total_read {
            loop {
                n = source.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                in_total_read = in_total_read + n;
                if in_total_read > total_written {
                    offset = total_written - (in_total_read - n);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_small_blocksize() {
        let mut cmd = ReplaceCommand {
            begin: 0,
            value: None,
        };
        let mut out: Vec<u8> = vec![];

        for bs in vec![1, 2, 3, 4, 10] {
            let mut rng = thread_rng();
            let len: usize = rng.gen_range(0, 30);
            let input: Vec<u8> = rng
                .sample_iter(rand::distributions::Standard)
                .take(len)
                .collect();

            for start in 0..=input.len() {
                let len: usize = rng.gen_range(0, 40);
                let text_to_replace: Vec<u8> = rng
                    .sample_iter(rand::distributions::Standard)
                    .take(len)
                    .collect();

                let mut exp = input[0..start].to_vec();
                exp.extend_from_slice(&text_to_replace);
                if input.len() > (start + len) {
                    exp.extend_from_slice(&input[start + len..]);
                }
                out.clear();
                cmd.begin = start;
                cmd.value = Some(text_to_replace);
                assert!(cmd.run(bs, &mut input.as_slice(), &mut out, None).is_ok());
                assert_eq!(exp, out);
            }
        }
    }

    #[test]
    fn test_big_blocksize() {
        let mut cmd = ReplaceCommand {
            begin: 0,
            value: None,
        };
        let mut out: Vec<u8> = vec![];

        for bs in vec![32, 64, 128, 512, 1024, 2048] {
            let mut rng = thread_rng();
            let len: usize = rng.gen_range(0, 10000);
            let input: Vec<u8> = rng
                .sample_iter(rand::distributions::Standard)
                .take(len)
                .collect();

            for start in 0..=input.len() {
                let len: usize = rng.gen_range(0, 2 * bs);
                let text_to_replace: Vec<u8> = rng
                    .sample_iter(rand::distributions::Standard)
                    .take(len)
                    .collect();

                let mut exp = input[0..start].to_vec();
                exp.extend_from_slice(&text_to_replace);
                if input.len() > (start + len) {
                    exp.extend_from_slice(&input[start + len..]);
                }
                out.clear();
                cmd.begin = start;
                cmd.value = Some(text_to_replace);
                assert!(cmd.run(bs, &mut input.as_slice(), &mut out, None).is_ok());
                assert_eq!(
                    exp,
                    out,
                    "exp.len: {}, out.len: {}; in.len: {}, replace.len: {}, start: {}",
                    exp.len(),
                    out.len(),
                    input.len(),
                    len,
                    start
                );
            }
        }
    }
}
