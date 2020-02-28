use clap::{value_t, ArgMatches};
use failure::{bail, Error};

pub struct AddCommand {
    begin: i64,
    value: Option<Vec<u8>>,
}
impl AddCommand {
    pub fn from_matches(m: &ArgMatches) -> Result<Self, Error> {
        let begin = value_t!(m, "begin", i64)?;
        let value = if let Ok(value) = value_t!(m, "value", String) {
            Some(value.as_bytes().to_vec())
        } else {
            None
        };

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
                offset = begin - (total_read - n);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_small_blocksize() {
        let mut cmd = AddCommand {
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

            for start in 0..input.len() + 1 {
                let len: usize = rng.gen_range(0, 40);
                let text_to_insert: Vec<u8> = rng
                    .sample_iter(rand::distributions::Standard)
                    .take(len)
                    .collect();

                let mut exp = input[0..start].to_vec();
                exp.extend_from_slice(&text_to_insert);
                exp.extend_from_slice(&input[start..]);
                out.clear();
                cmd.begin = start as i64;
                cmd.value = Some(text_to_insert);
                assert!(cmd.run(bs, &mut input.as_slice(), &mut out, None).is_ok());
                assert_eq!(exp, out);
            }
        }
    }

    #[test]
    fn test_big_blocksize() {
        let mut cmd = AddCommand {
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
                let to_insert: Vec<u8> = rng
                    .sample_iter(rand::distributions::Standard)
                    .take(len)
                    .collect();

                let mut exp = input[0..start].to_vec();
                exp.extend_from_slice(&to_insert);
                exp.extend_from_slice(&input[start..]);
                out.clear();
                cmd.begin = start as i64;
                cmd.value = Some(to_insert);
                assert!(cmd.run(bs, &mut input.as_slice(), &mut out, None).is_ok());
                assert_eq!(exp, out);
            }
        }
    }
}
