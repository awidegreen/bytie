use std::path::PathBuf;

use crate::command::{
    add::AddCommand, cut::CutCommand, delete::DeleteCommand, replace::ReplaceCommand, Command,
};
use anyhow::Result;
use clap::{Parser, Subcommand};

const DEFAULT_BLOCKSIZE: usize = 1024;

/// bytie - convenient byte stream manipulation
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Use output file instead of STDOUT
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Use different block-size when reading from source, default is 1024
    #[arg(short, long, default_value_t = DEFAULT_BLOCKSIZE )]
    pub blocksize: usize,

    /// Change file in-place, doesn't work with STDIN as input
    #[arg(short = 'I', long, default_value_t = false)]
    pub in_place: bool,

    /// Specify an input file, if not present, STDIN input is required (disables STDIN for sub-commands)
    #[arg(short, long)]
    pub input: Option<PathBuf>,

    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add bytes to a file/input
    Add(AddCommand),
    /// Add bytes to a file/input
    Replace(ReplaceCommand),
    /// Delete/Remove bytes from file/input
    Delete(DeleteCommand),
    /// Cut/extract bytes from file/input
    Cut(CutCommand),
}

impl Command for Commands {
    fn run(
        &self,
        blocksize: usize,
        source: &mut dyn std::io::Read,
        out: &mut dyn std::io::Write,
        input: Option<&mut dyn std::io::Read>,
    ) -> Result<()> {
        match self {
            Self::Add(c) => c.run(blocksize, source, out, input),
            Self::Cut(c) => c.run(blocksize, source, out, input),
            Self::Delete(c) => c.run(blocksize, source, out, input),
            Self::Replace(c) => c.run(blocksize, source, out, input),
        }
    }
}
