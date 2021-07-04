use clap::{load_yaml, App};
use image::DynamicImage;
use img_hash::HasherConfig;
use std::path::Path;

fn main() {
    let clap_def = load_yaml!("cli_def.yaml");
    let matches = App::from_yaml(clap_def).get_matches();

    println!("{:?}", matches);
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
