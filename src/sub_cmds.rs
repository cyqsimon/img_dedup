//! Each exported function in this module encapsulates
//! all the tasks necessary for a single subcommand.

use std::path::PathBuf;

use clap::ArgMatches;
use crossbeam_channel::Receiver;
use image::DynamicImage;

use crate::{
    file_loader::get_filename_unchecked,
    sub_ops::{filter_max_dist, pairwise_hash_dist, stream_hash},
};

pub fn hash_once(imgs_rx: Receiver<(PathBuf, DynamicImage)>, concurrency: usize, sub_matches: &ArgMatches) {
    // compute hashes
    let name_hash_pairs: Vec<_> = stream_hash(imgs_rx, concurrency, sub_matches)
        .unwrap() // sub_matches should satisfy arg requirements
        .into_iter()
        .map(|(path, hash)| (get_filename_unchecked(&path).to_string(), hash))
        .collect();

    // format and log
    const NAME_FMT_MAX_LEN: usize = 30; // file names longer than this get truncated

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

pub fn scan_duplicates(imgs_rx: Receiver<(PathBuf, DynamicImage)>, concurrency: usize, sub_matches: &ArgMatches) {
    // compute hashes
    let path_hash_pairs: Vec<_> = stream_hash(imgs_rx, concurrency, sub_matches).unwrap(); // sub_matches should satisfy arg requirements

    // compute pairwise hamming distances
    let pairwise_distances = pairwise_hash_dist(&path_hash_pairs, concurrency);

    // filter by threshold
    let mut similar_pairs = filter_max_dist(&pairwise_distances, sub_matches).unwrap(); // sub_matches should satisfy arg requirements

    // sort by distance and log each entry
    similar_pairs.sort_by_key(|(_, _, dist)| *dist);
    for &(p0, p1, dist) in similar_pairs {
        let n0 = get_filename_unchecked(p0);
        let n1 = get_filename_unchecked(p1);
        println!("  [{}] - [{}]  Distance: {}", n0, n1, dist);
    }
}
