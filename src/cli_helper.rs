use img_hash::HashAlg;

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

pub fn parse_algo(arg: &str) -> Result<HashAlg, String> {
    use HashAlg::*;
    match arg {
        "mean" => Ok(Mean),
        "h-gradient" => Ok(Gradient),
        "v-gradient" => Ok(VertGradient),
        "double-gradient" => Ok(DoubleGradient),
        "blockhash" => Ok(Blockhash),
        other => Err(format!("\"{}\" is not a valid hashing algorithm", other)),
    }
}
