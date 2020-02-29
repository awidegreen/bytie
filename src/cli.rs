use crate::defs;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub(crate) fn get_matches<'a>() -> ArgMatches<'a> {
    App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!("\n"))
        .about("bytie - convinient byte stream manipulation")
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
                .help("Change input file in-place. This does't work with STDIN as an input.")
                .short("i")
                .long(defs::IN_PLACE_P),
        )
        .arg(
            Arg::with_name("file")
                .help("Specify an input file, if not present, STDIN input is required (disables STDIN for subcommands)")
                .takes_value(true)
                .required(false),
        )
        .subcommand(
            SubCommand::with_name("add")
                .setting(AppSettings::AllowLeadingHyphen)
                .about("Add bytes to a file/input")
                .visible_alias("insert")
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
                .visible_alias("substitute")
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
                .visible_alias("remove")
                .arg(
                    Arg::with_name("position")
                        .help(
                            r##"Specify a position/range where the deletion shall be performed on.
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
                .visible_alias("extract")
                .arg(
                    Arg::with_name("position")
                        .help(
                            r##"Specify a position/range where a cut shall be performed.
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
