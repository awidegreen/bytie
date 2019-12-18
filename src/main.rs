use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use log::error;
mod command;
mod defs;
mod utils;

fn get_matches<'a>() -> ArgMatches<'a> {
    App::new("bytie")
        .version("0.1")
        .author("Armin Widegreen <bytie@widegreen.net>")
        .about("bytie - byte steam manipulate from command line")
        .settings(&[
            AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands,
            AppSettings::InferSubcommands,
        ])
        .arg(
            Arg::with_name(defs::OUTPUT_P)
                .help("Use output file instead of STDOUT")
                .short("o")
                .value_name("OUTPUT")
                .long("out"),
        )
        .arg(
            Arg::with_name(defs::BLOCKSIZE_P)
                .help(
                    format!(
                        "Use different blocksize when reading from the source, default {}",
                        defs::BLOCKSIZE
                    )
                    .as_str(),
                )
                .short("b")
                .value_name("BLOCKSIZE")
                .long(defs::BLOCKSIZE_P),
        )
        .arg(
            Arg::with_name(defs::IN_PLACE_P)
                .help("Change input file in place")
                .short("i")
                .long(defs::IN_PLACE_P),
        )
        .arg(
            Arg::with_name("file")
                .help("Specify an input file, if not present, STDIN input is required")
                .takes_value(true)
                .required(false),
        )
        .subcommand(
            SubCommand::with_name("add")
                .setting(AppSettings::AllowLeadingHyphen)
                .about("Add bytes to a file/input")
                .alias("insert")
                .arg(
                    Arg::with_name("begin")
                        .help(
                            r##"Specify where the data should be added.
This should be an integer, where -1 specifies then end of the file"##,
                        )
                        .allow_hyphen_values(true)
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("value")
                        .help(
                            "Input string that should be added, if not provided STDIN will be used",
                        )
                        .long("value")
                        .short("v")
                        .takes_value(true)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("replace")
                .about("Replace bytes of a file/input")
                .alias("substitute")
                .arg(
                    Arg::with_name("begin")
                        .help(r##"Specify where the replacement should start."##)
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("value")
                        .help(
                            "Input string that should be added, if not provided STDIN will be used (if it is not the in data)",
                        )
                        .long("value")
                        .short("v")
                        .takes_value(true)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("delete")
                .about("Delete/Remove bytes from file/input")
                .alias("remove")
                .arg(
                    Arg::with_name("position")
                        .help(
                            r##"Specify a position/range the deletion shall be performed on.
<begin>:<end>         Begin to end (inclusive), requires <end> > <begin>
<begin>+<length>      Begin plus <length bytes>, requires <length> > 0
<begin>               Begin to the end of input
                            "##,
                        )
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("cut")
                .about("Cut/extract bytes from file/input")
                .alias("extract")
                .arg(
                    Arg::with_name("position")
                        .help(
                            r##"Specify a position/range where the cut shall be performed on.
<begin>:<end>         Begin to end (inclusive), requires <end> > <begin>
<begin>+<length>      Begin plus <length bytes>, requires <length> > 0
<begin>               Begin to the end of input
                            "##,
                        )
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches()
}

fn main() {
    env_logger::init();
    let matches = get_matches();

    let runner = command::CommandRunner::from_matches(&matches).unwrap();

    if let Some(matches) = matches.subcommand_matches("delete") {
        let mut cmd = command::delete::DeleteCommand::from_matches(matches).unwrap();
        match runner.exec(&mut cmd) {
            Ok(_) => (),
            Err(x) => error!("{}", x),
        }
    }

    if let Some(matches) = matches.subcommand_matches("cut") {
        let mut cmd = command::cut::CutCommand::from_matches(matches).unwrap();
        match runner.exec(&mut cmd) {
            Ok(_) => (),
            Err(x) => error!("{}", x),
        }
    }

    if let Some(matches) = matches.subcommand_matches("add") {
        let mut cmd = command::add::AddCommand::from_matches(matches).unwrap();
        match runner.exec(&mut cmd) {
            Ok(_) => (),
            Err(x) => error!("{}", x),
        }
    }

    if let Some(matches) = matches.subcommand_matches("replace") {
        let mut cmd = command::replace::ReplaceCommand::from_matches(matches).unwrap();
        match runner.exec(&mut cmd) {
            Ok(_) => (),
            Err(x) => error!("{}", x),
        }
    }
}
