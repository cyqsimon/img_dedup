//! This module defines the CLI API via clap.

use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use regex::Regex;

use crate::cli_helper::parse_hash_size;

/// Build a clap app. Only call once.
pub fn build_app() -> App<'static, 'static> {
    let arg_algo = Arg::with_name("algorithm")
        .short("a")
        .long("algorithm")
        .takes_value(true)
        .possible_values(&["mean", "h-gradient", "v-gradient", "double-gradient", "blockhash"])
        .default_value("double-gradient")
        .help("Set an alternate hash algorithm (long help available)")
        .long_help(
            "Set an alternate hash algorithm\
        \nSee https://docs.rs/img_hash/latest/img_hash/enum.HashAlg.html for choices",
        );
    let arg_hash_size = Arg::with_name("hash-size")
        .short("s")
        .long("hash-size")
        .takes_value(true)
        .default_value("12,12")
        .validator(|arg| parse_hash_size(&arg).map(|_| ()))
        .help("Set a custom hash size (long help available)")
        .long_help(
            "Set a custom hash size\
            \nAccepts a single u32, or two comma-separated u32s (e.g. 20,16)\
            \nSee https://docs.rs/img_hash/latest/img_hash/struct.HasherConfig.html#hash-size \
            for value selection",
        );
    let arg_dist_threshold = Arg::with_name("threshold")
        .short("t")
        .long("threshold")
        .takes_value(true)
        .default_value("16")
        .validator(|arg| arg.parse::<u32>().map(|_| ()).map_err(|e| e.to_string()))
        .help("Hamming distance upper threshold (inclusive) (long help available")
        .long_help(
            "The minimum hamming distance for images to be considered similar (inclusive)\
            \nNote: the larger the hash size, the larger the hamming distances will generally become",
        );

    App::new("Image Deduplicator")
        .version(crate_version!())
        .author("Scheimong <28627918+cyqsimon@users.noreply.github.com>")
        .about("A command line program that finds and removes duplicated images using perceptual hashing")
        .settings(&[
            AppSettings::AllowNegativeNumbers,
            AppSettings::ArgRequiredElseHelp,
            AppSettings::DisableHelpSubcommand,
            AppSettings::InferSubcommands,
            AppSettings::SubcommandRequiredElseHelp,
        ])
        .arg(
            Arg::with_name("input_dir")
                .required(true)
                .index(1)
                .help("The directory to source input images from"),
        )
        .arg(
            Arg::with_name("input_filter")
                .short("f")
                .long("in_filter")
                .takes_value(true)
                .default_value(".*")
                .validator(|arg| Regex::new(&arg).map(|_| ()).map_err(|e| e.to_string()))
                .help("Only accept files that match the regex filter"),
        )
        .arg({
            // by default, use as many threads as the host has logical cores
            // create never-freed static str, see https://stackoverflow.com/a/30527289/5637701
            // need to do this, otherwise we've got lifetime problems
            let default_val: &'static str = Box::leak(num_cpus::get().to_string().into_boxed_str());
            Arg::with_name("concurrency")
                .short("c")
                .long("concurrency")
                .takes_value(true)
                .default_value(default_val)
                .validator(|arg| {
                    arg.parse::<usize>()
                        .map_err(|e| e.to_string())
                        .and_then(|th| (th != 0).then(|| ()).ok_or("Cannot specify 0 threads".into()))
                })
                .help("The number of threads to use for parallel computing")
        })
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the verbosity level of output. This is a repeated flag"),
        )
        .subcommand(
            SubCommand::with_name("hash")
                .about("Compute and show hashes for the input files")
                .arg(&arg_algo)
                .arg(&arg_hash_size),
        )
        .subcommand(
            SubCommand::with_name("scan-duplicates")
                .about("Scan the input files for duplicates and show them")
                .arg(&arg_algo)
                .arg(&arg_hash_size)
                .arg(&arg_dist_threshold),
        )
        .subcommand(
            SubCommand::with_name("move-duplicates")
                .about("Scan for duplicates, then move them to another directory")
                .arg(&arg_algo)
                .arg(&arg_hash_size)
                .arg(&arg_dist_threshold)
                .arg(
                    Arg::with_name("destination")
                        .required(true)
                        .index(1)
                        .help("The destination directory for duplicate files"),
                ),
        )
}
