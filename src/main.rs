mod cli;
mod command;
mod range;
mod utils;

use anyhow::Result;
use clap::Parser;
use command::CommandRunner;

fn main() -> Result<()> {
    env_logger::init();

    let mut cli = cli::Cli::parse();

    let runner = CommandRunner {
        blocksize: cli.blocksize,
        in_place: cli.in_place,
        out_file: cli.output,
        in_file: cli.input,
    };

    runner.exec(&mut cli.cmd)?;

    Ok(())
}
