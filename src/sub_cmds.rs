//! Each exported function in this module encapsulates
//! all the tasks necessary for a single subcommand.

use std::{collections::HashSet, path::PathBuf, process::exit};

use clap::ArgMatches;
use crossbeam_channel::Receiver;
use image::DynamicImage;
use img_hash::ImageHash;

use crate::{
    io::get_filename_unchecked,
    sub_ops::{filter_max_dist, log_pairwise_dists_sorted, move_all, pairwise_hash_dist, stream_hash},
};

/// Corresponds to subcommand `hash`.
pub fn hash_once(
    imgs_rx: Receiver<(PathBuf, DynamicImage)>,
    concurrency: usize,
    sub_matches: &ArgMatches,
) -> Vec<(PathBuf, ImageHash)> {
    // compute hashes
    let path_hash_pairs: Vec<_> = stream_hash(imgs_rx, concurrency, sub_matches).unwrap(); // sub_matches should satisfy arg requirements

    // format and log
    const NAME_FMT_MAX_LEN: usize = 30; // file names longer than this get truncated
    let name_hash_pairs: Vec<_> = path_hash_pairs
        .iter()
        .map(|(path, hash)| (get_filename_unchecked(&path).to_string(), hash))
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

    path_hash_pairs
}

/// Corresponds to subcommand `scan-duplicates`.
pub fn scan_duplicates(
    imgs_rx: Receiver<(PathBuf, DynamicImage)>,
    concurrency: usize,
    sub_matches: &ArgMatches,
) -> Vec<(PathBuf, PathBuf, u32)> {
    // compute hashes
    let path_hash_pairs: Vec<_> = stream_hash(imgs_rx, concurrency, sub_matches).unwrap(); // sub_matches should satisfy arg requirements

    // compute pairwise hamming distances
    let pairwise_distances = pairwise_hash_dist(&path_hash_pairs, concurrency);

    // filter by threshold
    let similar_pairs = filter_max_dist(&pairwise_distances, sub_matches).unwrap(); // sub_matches should satisfy arg requirements

    // log each entry
    log_pairwise_dists_sorted(&similar_pairs);

    // ref -> owned
    similar_pairs
        .into_iter()
        .map(|&(p0, p1, d)| (p0.into(), p1.into(), d))
        .collect()
}

/// Corresponds to subcommand `move-duplicates`.
pub fn move_duplicates(imgs_rx: Receiver<(PathBuf, DynamicImage)>, concurrency: usize, sub_matches: &ArgMatches) {
    use std::iter::once;

    // compute hashes
    let path_hash_pairs: Vec<_> = stream_hash(imgs_rx, concurrency, sub_matches).unwrap(); // sub_matches should satisfy arg requirements

    // compute pairwise hamming distances
    let pairwise_distances = pairwise_hash_dist(&path_hash_pairs, concurrency);

    // filter by threshold
    let similar_pairs = filter_max_dist(&pairwise_distances, sub_matches).unwrap(); // sub_matches should satisfy arg requirements

    // move all duplicates
    if similar_pairs.len() == 0 {
        println!("No duplicate images found");
        return;
    }
    let all_files: HashSet<_> = similar_pairs
        .into_iter()
        .flat_map(|&(p0, p1, _)| once(p0).chain(once(p1)))
        .collect();
    if let Err(e) = move_all(&all_files, sub_matches) {
        println!("Failed to move duplicate images: {:?}", e);
        exit(1);
    }
}
