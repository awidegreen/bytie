pub mod add;
pub mod cut;
pub mod delete;
pub mod replace;
use crate::defs;
use clap::{value_t, ArgMatches};
use failure::{bail, Error};
use std::fs::OpenOptions;

pub trait Command {
    fn run(
        &self,
        blocksize: usize,
        source: &mut dyn std::io::Read,
        out: &mut dyn std::io::Write,
        input: Option<&mut dyn std::io::Read>,
    ) -> Result<(), Error>;
}

pub struct CommandRunner {
    blocksize: usize,
    in_place: bool,
    out_file: Option<String>,
    in_file: Option<String>,
}

impl CommandRunner {
    pub fn from_matches(matches: &ArgMatches) -> Result<Self, Error> {
        let blocksize = value_t!(matches, defs::BLOCKSIZE_P, usize).unwrap_or(defs::BLOCKSIZE);
        let in_place = matches.is_present(defs::IN_PLACE_P);
        let out_file = value_t!(matches, defs::OUTPUT_P, String).ok();
        let in_file = value_t!(matches, "file", String).ok();

        Ok(CommandRunner {
            blocksize,
            in_place,
            out_file,
            in_file,
        })
    }

    fn exec_impl(
        &self,
        src: &mut dyn std::io::Read,
        input: Option<&mut dyn std::io::Read>,
        command: &mut impl Command,
    ) -> Result<(), Error> {
        if let Some(ref fname) = self.out_file {
            let mut f = OpenOptions::new().write(true).create(true).open(fname)?;
            command.run(self.blocksize, src, &mut f, input)?;
        } else if self.in_place {
            if let Some(ref file) = self.in_file {
                let mut tmp_f = tempfile::NamedTempFile::new()?;
                command.run(self.blocksize, src, &mut tmp_f, input)?;
                std::fs::copy(tmp_f, file)?;
            } else {
                bail!("'in-place' requires an input file");
            }
        } else {
            command.run(self.blocksize, src, &mut std::io::stdout(), input)?;
        }

        Ok(())
    }

    pub fn exec(&self, command: &mut impl Command) -> Result<(), Error> {
        if let Some(in_file) = &self.in_file {
            let p = std::path::Path::new(&in_file);
            if !p.exists() {
                bail!("{} does not exists!", in_file);
            }
            let mut f = std::fs::File::open(p)?;

            if atty::isnt(atty::Stream::Stdin) {
                self.exec_impl(&mut f, Some(&mut std::io::stdin()), command)
            } else {
                self.exec_impl(&mut f, None, command)
            }
        } else {
            if atty::isnt(atty::Stream::Stdin) {
                self.exec_impl(&mut std::io::stdin(), None, command)
            } else {
                bail!("Some source is required, either <FILE> or STDIN")
            }
        }
    }
}
