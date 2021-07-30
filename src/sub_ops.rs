//! Each subcommand is composed of a chain of tasks,
//! many of which are common among multiple subcommands.
//! Therefore it makes sense for those tasks to be discrete,
//! in order to avoid code duplication.
//!
//! Each exported function in this module performs one said task,
//! and prints all relevant info to the console.

use std::path::{Path, PathBuf};

use clap::ArgMatches;
use crossbeam_channel::{unbounded, Receiver};
use image::DynamicImage;
use img_hash::ImageHash;

use crate::{
    cli_helper::{parse_algo, parse_hash_size},
    compute::{calc_hashes, calc_pair_dist},
};

pub fn stream_hash(
    imgs_rx: Receiver<(PathBuf, DynamicImage)>,
    concurrency: usize,
    sub_matches: &ArgMatches,
) -> Result<Vec<(PathBuf, ImageHash)>, String> {
    println!("Computing perceptual hash...");

    // get algorithm option
    let algo = parse_algo(
        sub_matches
            .value_of("algorithm")
            .ok_or_else(|| "algorithm not specified")?,
    )
    .unwrap(); // validation provided by clap

    // get hash size option
    let hash_size = parse_hash_size(
        sub_matches
            .value_of("hash-size")
            .ok_or_else(|| "hash-size not specified")?,
    )
    .unwrap(); // validation provided by clap

    // create a unified reply channel for worker threads
    let (hashes_tx, hashes_rx) = unbounded();
    // run calculations
    calc_hashes(imgs_rx, hashes_tx, concurrency, algo, hash_size);
    // hash reply channel buffer => vec
    let path_hash_pairs: Vec<_> = hashes_rx.into_iter().collect();

    println!(
        "Finished computing perceptual hash for {} image(s)",
        path_hash_pairs.len()
    );

    Ok(path_hash_pairs)
}

pub fn pairwise_hash_dist(path_hash_pairs: &[(PathBuf, ImageHash)], concurrency: usize) -> Vec<(&Path, &Path, u32)> {
    println!("Computing pairwise hamming distances...");

    // run calculations
    let pairwise_distances = calc_pair_dist(&path_hash_pairs, concurrency);

    println!(
        "Finished computing hamming distances for {} pairs",
        pairwise_distances.len()
    );

    pairwise_distances
}

pub fn filter_max_dist<'a>(
    pairwise_distances: &'a [(&Path, &Path, u32)],
    sub_matches: &ArgMatches,
) -> Result<Vec<&'a (&'a Path, &'a Path, u32)>, String> {
    println!("Filtering pairwise hamming distances...");

    // get threshold options
    let threshold = sub_matches
        .value_of("threshold")
        .ok_or_else(|| "threshold not specified")?
        .parse::<u32>()
        .unwrap(); // u32 parse validated by clap

    // filter
    let similar_pairs: Vec<_> = pairwise_distances
        .into_iter()
        .filter(|(_, _, dist)| *dist <= threshold)
        .collect();

    println!(
        "Found {} similar pair(s) with a hamming distance of ≤{}",
        similar_pairs.len(),
        threshold
    );

    Ok(similar_pairs)
}
