use anyhow::{anyhow, Error};
use bytesize::ByteSize;
use clap::Args;

use crate::{range::RANGE_HELP, utils::parse_pos};

fn parse_to_vec(v: &str) -> std::result::Result<Vec<u8>, std::io::Error> {
    Ok(v.as_bytes().to_vec())
}

#[derive(Args, Debug)]
#[command(name = "replace", visible_alias = "substitute", after_help = RANGE_HELP)]
pub(crate) struct ReplaceCommand {
    /// Specify where the replacement should start.
    ///
    /// The value can be integer specifying the byte position.
    ///
    /// Instead of providing a number, a human readable byte size string can be
    /// used, indicating the byte position to start. Example: 3B, 4Kb, 4KiB
    ///
    /// If no value is provided, the data will be appended to the input stream.
    #[arg(value_parser = clap::builder::ValueParser::new(parse_pos))]
    pub begin: Option<ByteSize>,

    /// Input string that should be added, if not provided STDIN will be used
    #[arg(short, long, value_parser = clap::builder::ValueParser::new(parse_to_vec))]
    pub value: Option<Vec<u8>>,
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
            return Err(anyhow!("Well, as no <VALUE> input parameter has been provided, some input should be provided by STDIN."));
        }

        let mut buffer = vec![0; blocksize];
        let mut in_total_read = 0;
        let mut offset = 0;
        let mut n;
        let mut total_written = 0;

        let begin = if let Some(begin) = self.begin {
            begin.as_u64() as usize
        } else {
            std::usize::MAX
        };

        // read from source until begin position
        loop {
            n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            in_total_read += n;
            if in_total_read > begin {
                offset = begin - (in_total_read - n);
                out.write_all(&buffer[0..offset])?;
                total_written += offset;
                break;
            } else {
                out.write_all(&buffer[0..n])?;
                total_written += n;
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
                out.write_all(&b[0..n])?;
                written += n;
            }
            written
        } else if let Some(value) = &self.value {
            out.write_all(value)?;
            value.len()
        } else {
            return Err(anyhow!(
                "No STDIN nor any <VALUE> has been provided, unable to proceed."
            ));
        };

        // if input data (replace with) is shorter than the block that has been
        // read before, make sure the current block is finished before reading
        // next block from source.
        if offset <= n && written < (n - offset) {
            offset += written;
            out.write_all(&buffer[offset..n])?;
        }
        total_written += written;

        if total_written > in_total_read {
            loop {
                n = source.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                in_total_read += n;

                match in_total_read.cmp(&total_written) {
                    std::cmp::Ordering::Greater => {
                        offset = total_written - (in_total_read - n);
                        out.write_all(&buffer[offset..n])?;
                        break;
                    }
                    std::cmp::Ordering::Equal => break,
                    _ => (),
                }
            }
        }
        loop {
            let n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            out.write_all(&buffer[0..n])?;
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
            begin: Some(Default::default()),
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
                cmd.begin = Some(ByteSize::b(start as u64));
                cmd.value = Some(text_to_replace);
                assert!(cmd.run(bs, &mut input.as_slice(), &mut out, None).is_ok());
                assert_eq!(exp, out);
            }
        }
    }

    #[test]
    fn test_big_blocksize() {
        let mut cmd = ReplaceCommand {
            begin: Some(Default::default()),
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
                cmd.begin = Some(ByteSize::b(start as u64));
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
