use clap::{load_yaml, App};
use image::DynamicImage;
use img_dedup::load_in;
use img_hash::HasherConfig;
use regex::Regex;
use std::{path::Path, process::exit};

fn main() {
    let clap_def = load_yaml!("cli_def.yaml");
    let matches = App::from_yaml(clap_def).get_matches();

    let in_dir = matches.value_of("input_dir").unwrap(); // arg is required
    let in_filter_str = matches.value_of("input_filter").unwrap_or(""); // "" matches all
    let in_filter_regex = Regex::new(in_filter_str).unwrap_or_else(|e| {
        println!("The provided input filter is not a valid regex.");
        println!("{:?}", e);
        exit(1)
    });

    let load_res = load_in(Path::new(in_dir), in_filter_regex);
    if let Err(e) = load_res {
        println!("Failed to open the input directory.");
        println!("{:?}", e);
        exit(1);
    }
    let load_vec = load_res.unwrap();

    let err_count = load_vec.iter().filter(|res| res.is_err()).count();
    if err_count != 0 {
        println!("Failed to load {} file(s) due to IO error.", err_count);
    }

    let imgs: Vec<_> = load_vec.into_iter().filter_map(|res| res.ok()).collect();
    println!("Successfully loaded {} image file(s).", imgs.len());
}

fn calc_hash(imgs: &[(&Path, &DynamicImage)]) {
    const NAME_FMT_MAX_LEN: usize = 30;

    let hasher = HasherConfig::new().to_hasher();
    let name_hash_pairs: Vec<_> = imgs
        .into_iter()
        .map(|&(path, img)| {
            (
                path.file_name()
                    .expect("File \"..\" encountered unexpectedly.")
                    .to_str()
                    .expect("Bad file name (non-UTF8) encountered unexpectedly."),
                hasher.hash_image(img),
            )
        })
        .collect();
    let name_fmt_len = name_hash_pairs
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0)
        .min(NAME_FMT_MAX_LEN);
    for (name, hash) in name_hash_pairs {
        let name_truncated_braced = format!("[{:.max_len$}]", name, max_len = NAME_FMT_MAX_LEN);
        println!(
            "  Img: {:<fmt_len$}  Hash: [{}]",
            name_truncated_braced,
            hash.to_base64(),
            fmt_len = name_fmt_len + 2
        );
    }
}
