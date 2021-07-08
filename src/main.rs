mod clap_def;

use image::DynamicImage;
use img_dedup::{get_filename_unchecked, load_in};
use img_hash::HasherConfig;
use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
use std::{path::Path, process::exit};

use crate::clap_def::build_app;

fn main() {
    let clap_matches = build_app().get_matches();

    let in_dir = clap_matches.value_of("input_dir").unwrap(); // arg is required
    let in_filter_regex = clap_matches
        .value_of("input_filter")
        .map(|rgx_str| Regex::new(rgx_str).unwrap()) // regex validated by clap
        .unwrap_or(Regex::new(".*").unwrap()); // ".*" matches everything

    // load all files in directory, optionally using filter
    println!(
        "Loading files in [{}] with regex filter [/{}/].",
        in_dir,
        in_filter_regex.as_str()
    );
    let load_res = load_in(Path::new(in_dir), &in_filter_regex);

    if let Err(e) = load_res {
        println!("Failed to open the input directory.");
        println!("{:?}", e);
        exit(1);
    }
    let load_vec = load_res.unwrap(); // Err case guarded

    // log unsuccessful
    let err_count = load_vec.iter().filter(|res| res.is_err()).count();
    if err_count != 0 {
        println!("Failed to load {} file(s) due to IO error.", err_count);
    }

    // collect and log successful
    let imgs: Vec<_> = load_vec.into_iter().filter_map(|res| res.ok()).collect();
    println!("Successfully loaded {} image file(s).", imgs.len());

    // generate by-ref vector
    let imgs_refs: Vec<_> = imgs.iter().map(|(path, img)| (path.as_ref(), img)).collect();

    // dispatch task to subcmds
    match clap_matches.subcommand() {
        ("compute-hash", Some(_sub_matches)) => compute_hash(&imgs_refs),
        ("scan-duplicates", Some(sub_matches)) => {
            let threshold = sub_matches
                .value_of("threshold")
                .unwrap() // clap provides default
                .parse::<u32>()
                .unwrap(); // u32 is validated by clap
            scan_duplicates(&imgs_refs, threshold);
        }
        _ => unreachable!(), // cases should always cover all defined subcmds; subcmds required
    };
}

fn compute_hash(imgs: &[(&Path, &DynamicImage)]) {
    const NAME_FMT_MAX_LEN: usize = 30;

    // compute hashes
    println!("Computing perceptual hash for {} image(s)...", imgs.len());
    let name_hash_pairs: Vec<_> = imgs
        .par_iter()
        .map_init(
            || HasherConfig::new().to_hasher(),
            |hasher, &(path, img)| (get_filename_unchecked(path), hasher.hash_image(img)),
        )
        .collect();
    println!(
        "Finished computing perceptual hash for {} image(s).",
        name_hash_pairs.len()
    );

    // format and log
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

fn scan_duplicates(imgs: &[(&Path, &DynamicImage)], threshold: u32) {
    // compute hashes
    println!("Computing perceptual hash for {} image(s)...", imgs.len());
    let path_hash_pairs: Vec<_> = imgs
        .par_iter()
        .map_init(
            || HasherConfig::new().to_hasher(),
            |hasher, &(path, img)| (path, hasher.hash_image(img)),
        )
        .collect();
    println!(
        "Finished computing perceptual hash for {} image(s).",
        path_hash_pairs.len()
    );

    // compute pairwise hamming distances
    println!(
        "Computing pairwise hamming distance for {} image pair(s)...",
        path_hash_pairs.len() * (path_hash_pairs.len() - 1) / 2
    );
    let pairwise_distances: Vec<_> = path_hash_pairs
        .into_iter()
        .tuple_combinations::<(_, _)>()
        .map(|((path0, hash0), (path1, hash1))| (path0, path1, hash0.dist(&hash1)))
        .collect();
    println!(
        "Finished computing hamming distance for {} image pair(s).",
        pairwise_distances.len()
    );

    // log summary
    let mut similar_pairs: Vec<_> = pairwise_distances
        .iter()
        .filter_map(|&(path0, path1, dist)| {
            (dist <= threshold).then(|| (get_filename_unchecked(path0), get_filename_unchecked(path1), dist))
        })
        .collect();
    println!(
        "Found {} similar pair(s) with a hamming distance of ≤{}.",
        similar_pairs.len(),
        threshold
    );

    // sort by distance and log each entry
    similar_pairs.sort_by_key(|(_, _, dist)| *dist);
    for (name0, name1, dist) in similar_pairs {
        println!("  [{}] - [{}]  Distance: {}", name0, name1, dist);
    }
}
