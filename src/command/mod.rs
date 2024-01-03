pub mod add;
pub mod cut;
pub mod delete;
pub mod replace;
use anyhow::{anyhow, Result};
use std::{fs::OpenOptions, path::PathBuf};

pub trait Command {
    fn run(
        &self,
        blocksize: usize,
        source: &mut dyn std::io::Read,
        out: &mut dyn std::io::Write,
        input: Option<&mut dyn std::io::Read>,
    ) -> Result<()>;
}

pub struct CommandRunner {
    pub blocksize: usize,
    pub in_place: bool,
    pub out_file: Option<PathBuf>,
    pub in_file: Option<PathBuf>,
}

impl CommandRunner {
    fn exec_impl(
        &self,
        src: &mut dyn std::io::Read,
        input: Option<&mut dyn std::io::Read>,
        command: &mut impl Command,
    ) -> Result<()> {
        if let Some(ref fname) = self.out_file {
            let mut f = OpenOptions::new().write(true).create(true).open(fname)?;
            command.run(self.blocksize, src, &mut f, input)?;
        } else if self.in_place {
            if let Some(ref file) = self.in_file {
                let mut tmp_f = tempfile::NamedTempFile::new()?;
                command.run(self.blocksize, src, &mut tmp_f, input)?;
                std::fs::copy(tmp_f, file)?;
            } else {
                return Err(anyhow!("'in-place' requires an input file"));
            }
        } else {
            command.run(self.blocksize, src, &mut std::io::stdout(), input)?;
        }

        Ok(())
    }

    pub fn exec(&self, command: &mut impl Command) -> Result<()> {
        if let Some(in_file) = &self.in_file {
            let p = std::path::Path::new(&in_file);
            if !p.exists() {
                return Err(anyhow::anyhow!("{:?} does not exists!", in_file));
            }
            let mut f = std::fs::File::open(p)?;

            if atty::isnt(atty::Stream::Stdin) {
                self.exec_impl(&mut f, Some(&mut std::io::stdin()), command)
            } else {
                self.exec_impl(&mut f, None, command)
            }
        } else if atty::isnt(atty::Stream::Stdin) {
            self.exec_impl(&mut std::io::stdin(), None, command)
        } else {
            Err(anyhow!("Some source is required, either <FILE> or STDIN"))
        }
    }
}
