use anyhow::{anyhow, Error};
use bytesize::ByteSize;
use clap::Args;

use crate::utils::{parse_pos, parse_to_vec};

#[derive(Args, Debug)]
#[command(visible_alias = "insert")]
pub struct AddCommand {
    /// Input string that should be added, if not provided STDIN will be used
    #[arg(short, long, default_value=None, value_parser = parse_to_vec)]
    // std::vec::Vec is required, see
    // https://docs.rs/clap/latest/clap/_derive/index.html#arg-types
    pub value: Option<std::vec::Vec<u8>>,

    /// Specify where the data should be added.
    ///
    /// The value can be integer specifying the byte position.
    ///
    /// Instead of providing a number, a human readable byte size string can be
    /// used, indicating the byte position to start. Example: 3B, 4Kb, 4KiB
    ///
    /// If no value is provided, the data will be appended to the input stream.
    #[arg(value_parser = parse_pos)]
    pub begin: Option<ByteSize>,
}

impl crate::command::Command for AddCommand {
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
        let mut total_read = 0;
        let mut offset = 0;
        let mut n;

        let begin = if let Some(begin) = self.begin {
            begin.as_u64() as usize
        } else {
            std::usize::MAX
        };

        loop {
            n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            total_read += n;
            if total_read > begin {
                offset = begin - (total_read - n);
                out.write_all(&buffer[0..offset])?;
                break;
            } else {
                out.write_all(&buffer[0..n])?;
            }
        }

        if let Some(input) = input {
            let mut buffer = vec![0; blocksize];
            loop {
                let n = input.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                out.write_all(&buffer[0..n])?;
            }
        } else if let Some(value) = &self.value {
            out.write_all(value.as_ref())?;
        } else {
            return Err(anyhow!("No stdin nor any value has been provided"));
        }

        if offset <= n {
            out.write_all(&buffer[offset..n])?;
        }

        while n != 0 {
            n = source.read(&mut buffer)?;
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
    fn test_add_to_end() {
        let cmd = AddCommand {
            begin: Some(ByteSize::b(std::u64::MAX)),
            value: Some(vec![3, 4, 5]),
        };
        let input = vec![0, 1, 2];
        let exp = vec![0, 1, 2, 3, 4, 5];

        for bs in vec![1, 2, 3, 4, 10] {
            let mut out: Vec<u8> = vec![];
            assert!(cmd.run(bs, &mut input.as_slice(), &mut out, None).is_ok());
            assert_eq!(exp, out);
        }
    }

    #[test]
    fn test_small_blocksize() {
        let mut cmd = AddCommand {
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
                let text_to_insert: Vec<u8> = rng
                    .sample_iter(rand::distributions::Standard)
                    .take(len)
                    .collect();

                let mut exp = input[0..start].to_vec();
                exp.extend_from_slice(&text_to_insert);
                exp.extend_from_slice(&input[start..]);
                out.clear();
                cmd.begin = Some(ByteSize::b(start as u64));
                cmd.value = Some(text_to_insert);
                assert!(cmd.run(bs, &mut input.as_slice(), &mut out, None).is_ok());
                assert_eq!(exp, out);
            }
        }
    }

    #[test]
    fn test_big_blocksize() {
        let mut cmd = AddCommand {
            begin: Some(ByteSize::b(0)),
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
                let to_insert: Vec<u8> = rng
                    .sample_iter(rand::distributions::Standard)
                    .take(len)
                    .collect();

                let mut exp = input[0..start].to_vec();
                exp.extend_from_slice(&to_insert);
                exp.extend_from_slice(&input[start..]);
                out.clear();
                cmd.begin = Some(ByteSize::b(start as u64));
                cmd.value = Some(to_insert);
                assert!(cmd.run(bs, &mut input.as_slice(), &mut out, None).is_ok());
                assert_eq!(exp, out);
            }
        }
    }
}
