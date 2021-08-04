//! This module contains functions that help with parsing and validation
//! of complex arguments provided by the user via CLI.

use img_hash::HashAlg;

/// The `hash-size` argument can be provided in two ways:
/// - either a single u32 (e.g. `24`, equivalent to `24,24`),
/// - or a pair of u32s separated by comma (e.g. `32,24`).
///
/// This function parses and validates both cases,
/// even tolerating extraneous white spaces.
pub fn parse_hash_size(arg: &str) -> Result<(u32, u32), String> {
    fn parse_u32_nonzero(num: &str) -> Result<u32, String> {
        match num.parse::<u32>() {
            Ok(0) => Err("Hash size cannot be 0".to_string()),
            Err(e) => Err(format!("{}: \"{}\"", e.to_string(), num)),
            Ok(v) => Ok(v),
        }
    }

    let arg_l: Vec<_> = arg.split(',').map(|s| s.trim()).collect();
    match arg_l.len() {
        1 => parse_u32_nonzero(arg_l[0]).map(|n| (n, n)),
        2 => {
            let n0 = parse_u32_nonzero(arg_l[0])?;
            let n1 = parse_u32_nonzero(arg_l[1])?;
            Ok((n0, n1))
        }
        _ => Err(format!("Too many comma-separated values: \"{}\"", &arg)),
    }
}

/// This function parses the name of the selected algorithm
/// into its corresponding enum variant.
///
/// If you are adding more algorithms in the future,
/// remember to update `clap`'s possible values in [clap_def](crate::clap_def).
pub fn parse_algo(arg: &str) -> Result<HashAlg, String> {
    use HashAlg::*;
    match arg {
        "mean" => Ok(Mean),
        "h-gradient" => Ok(Gradient),
        "v-gradient" => Ok(VertGradient),
        "double-gradient" => Ok(DoubleGradient),
        "blockhash" => Ok(Blockhash),
        other => Err(format!("\"{}\" is not a supported hashing algorithm", other)),
    }
}
