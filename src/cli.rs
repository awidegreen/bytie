use crate::defs;
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

static POS_HELP: &str =
    "Specifies a position and range/count for the operation, see POSITION section";
static POS_HELP_SEC: &str = "POSITION:
\tSpecify a position/range where the deletion shall be performed on.
\t<begin>\t\t  Begin to the end of input
\t<begin>:<end>\t  Begin to end (exclusive), requires <end> > <begin>
\t\t\t  Example: 'foobar', 0:2 == 'fo' or 3:5 == 'ba'
\t<begin>:=<end>\t  Begin to end (inclusive), requires <end> > <begin>
\t\t\t  Example: 'foobar', 0:=2 == 'foo' or 3:=5 == 'bar'
\t<begin>+<count>\t  Begin plus <count> (exclusive), requires <count> > 0.
\t\t\t  The length includes the begin position: 0+10 is 10 bytes, from 0..9 (same as 0:9)
";

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
This should be an integer, where -1 specifies then end of the file/stream"##,
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
                .after_help(POS_HELP_SEC)
                .arg(
                    Arg::with_name("position")
                        .help(POS_HELP)
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("cut")
                .about("Cut/extract bytes from file/input")
                .visible_alias("extract")
                .after_help(POS_HELP_SEC)
                .arg(
                    Arg::with_name("position")
                        .help(POS_HELP)
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches()
}
