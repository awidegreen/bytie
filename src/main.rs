mod cli;
mod command;
mod defs;
mod utils;
use clap::ArgMatches;

fn main() {
    env_logger::init();
    let matches = cli::get_matches();

    println!("matches: {:?}", matches);

    let runner = command::CommandRunner::from_matches(&matches).unwrap();

    let exit_code = match matches.subcommand() {
        ("delete", Some(m)) => match command::delete::DeleteCommand::from_matches(m) {
            Ok(mut cmd) => match runner.exec(&mut cmd) {
                Ok(_) => exitcode::OK,
                Err(x) => {
                    eprintln!("{}", x);
                    exitcode::SOFTWARE
                }
            },
            Err(x) => {
                eprintln!("{}", x);
                exitcode::USAGE
            }
        },
        ("cut", Some(m)) => match command::cut::CutCommand::from_matches(m) {
            Ok(mut cmd) => match runner.exec(&mut cmd) {
                Ok(_) => exitcode::OK,
                Err(x) => {
                    eprintln!("{}", x);
                    exitcode::SOFTWARE
                }
            },
            Err(x) => {
                eprintln!("{}", x);
                exitcode::USAGE
            }
        },
        ("add", Some(m)) => match command::add::AddCommand::from_matches(m) {
            Ok(mut cmd) => match runner.exec(&mut cmd) {
                Ok(_) => exitcode::OK,
                Err(x) => {
                    eprintln!("{}", x);
                    exitcode::SOFTWARE
                }
            },
            Err(x) => {
                eprintln!("{}", x);
                exitcode::USAGE
            }
        },
        ("replace", Some(m)) => match command::replace::ReplaceCommand::from_matches(m) {
            Ok(mut cmd) => match runner.exec(&mut cmd) {
                Ok(_) => exitcode::OK,
                Err(x) => {
                    eprintln!("{}", x);
                    exitcode::SOFTWARE
                }
            },
            Err(x) => {
                eprintln!("{}", x);
                exitcode::USAGE
            }
        },
        _ => unreachable!(),
    };

    std::process::exit(exit_code);
}
