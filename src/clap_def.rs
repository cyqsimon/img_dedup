use clap::{crate_version, App, AppSettings, Arg, SubCommand};

pub fn build_app() -> App<'static, 'static> {
    App::new("Image Deduplicator")
        .version(crate_version!())
        .author("Scheimong <28627918+cyqsimon@users.noreply.github.com>")
        .about("A command line program that finds and removes duplicated images using perceptual hashing.")
        .settings(&[
            AppSettings::ArgRequiredElseHelp,
            AppSettings::DisableHelpSubcommand,
            AppSettings::InferSubcommands,
            AppSettings::SubcommandRequiredElseHelp,
        ])
        .arg(
            Arg::with_name("input_dir")
                .required(true)
                .index(1)
                .help("The directory to source input files from"),
        )
        .arg(
            Arg::with_name("input_filter")
                .short("f")
                .long("in_filter")
                .takes_value(true)
                .help("Only accept files that match the regex filter. Default: \"\" (match all)"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the verbosity level of output. This is a repeated flag"),
        )
        .subcommand(SubCommand::with_name("compute-hash").about("Compute and show hashes for the input files"))
        .subcommand(
            SubCommand::with_name("scan-duplicates")
                .about("Scan the input files for duplicates without removing anything")
                .arg(
                    Arg::with_name("threshold")
                        .short("t")
                        .long("threshold")
                        .takes_value(true)
                        .default_value("12")
                        .help("The minimum hamming distance for images to be considered similar"),
                ),
        )
}
