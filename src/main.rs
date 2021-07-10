mod clap_def;
mod file_loader;
mod stream_compute;

use crossbeam_channel::{bounded, unbounded, Receiver};
use image::DynamicImage;
use itertools::Itertools;
use regex::Regex;
use std::{
    fs::read_dir,
    path::{Path, PathBuf},
    process::exit,
    thread,
    time::Duration,
};

use crate::{
    clap_def::build_app,
    file_loader::{get_filename_unchecked, load_in},
    stream_compute::calc_hashes,
};

fn main() {
    let clap_matches = build_app().get_matches();

    // create single-producer, multiple-consumer channel
    let (imgs_tx, imgs_rx) = bounded(128);

    // get input options
    let in_dir = clap_matches.value_of("input_dir").unwrap(); // arg is required
    let in_filter_regex = clap_matches
        .value_of("input_filter")
        .map(|rgx_str| Regex::new(rgx_str).unwrap()) // regex validated by clap
        .unwrap_or(Regex::new(".*").unwrap()); // ".*" matches everything

    // opening imgs_dir outside of thread makes for easier code logic
    let opened_imgs_dir = read_dir(Path::new(in_dir)).unwrap_or_else(|e| {
        println!("Failed to open the input directory: {:?}", e);
        exit(1);
    });

    // start imgs loading (single producer)
    println!(
        "Loading files in [{}] with regex filter [/{}/]",
        in_dir,
        in_filter_regex.as_str()
    );
    thread::spawn(move || {
        load_in(imgs_tx, opened_imgs_dir, &in_filter_regex);
    });

    // spawn image loader monitor daemon
    let imgs_rx_monitor = imgs_rx.clone();
    let (monitor_kill_tx, monitor_kill_rx) = bounded(0);
    thread::spawn(move || loop {
        let queue_len = imgs_rx_monitor.len();
        if queue_len > 0 {
            println!(
                "IO loading images faster than we can hash; currently {} in queue",
                queue_len
            );
        }
        // sleep for 5 seconds total, but check every 100ms inbetween
        for _ in 0..50 {
            if let Ok(_) = monitor_kill_rx.try_recv() {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    // dispatch task to subcmds
    match clap_matches.subcommand() {
        ("compute-hash", Some(_sub_matches)) => compute_hash(imgs_rx),
        ("scan-duplicates", Some(sub_matches)) => {
            let threshold = sub_matches
                .value_of("threshold")
                .unwrap() // clap provides default
                .parse::<u32>()
                .unwrap(); // u32 is validated by clap
            scan_duplicates(imgs_rx, threshold);
        }
        _ => unreachable!(), // cases should always cover all defined subcmds; subcmds required
    };

    // stop monitoring daemon
    monitor_kill_tx
        .send(())
        .expect("Image loader monitor daemon failed unexpectedly");
}

fn compute_hash(imgs_rx: Receiver<(PathBuf, DynamicImage)>) {
    const NAME_FMT_MAX_LEN: usize = 30; // file names longer than this get truncated

    println!("Computing perceptual hash...");
    // create a unified reply channel for worker threads
    let (hashes_tx, hashes_rx) = unbounded();

    calc_hashes(imgs_rx, hashes_tx, num_cpus::get());

    // hash reply channel buffer => vec
    let name_hash_pairs: Vec<_> = hashes_rx
        .into_iter()
        .map(|(path, hash)| (get_filename_unchecked(&path).to_string(), hash))
        .collect();
    println!(
        "Finished computing perceptual hash for {} image(s)",
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

fn scan_duplicates(imgs_rx: Receiver<(PathBuf, DynamicImage)>, threshold: u32) {
    // compute hashes
    println!("Computing perceptual hash...");
    // create a unified reply channel for worker threads
    let (hashes_tx, hashes_rx) = unbounded();

    calc_hashes(imgs_rx, hashes_tx, num_cpus::get());

    // hash reply channel buffer => vec
    let path_hash_pairs: Vec<_> = hashes_rx.into_iter().collect();
    println!(
        "Finished computing perceptual hash for {} image(s)",
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
        "Finished computing hamming distance for {} image pair(s)",
        pairwise_distances.len()
    );

    // log summary
    let mut similar_pairs: Vec<_> = pairwise_distances
        .iter()
        .filter_map(|(path0, path1, dist)| {
            (dist <= &threshold).then(|| (get_filename_unchecked(path0), get_filename_unchecked(path1), dist))
        })
        .collect();
    println!(
        "Found {} similar pair(s) with a hamming distance of ≤{}",
        similar_pairs.len(),
        threshold
    );

    // sort by distance and log each entry
    similar_pairs.sort_by_key(|(_, _, dist)| *dist);
    for (name0, name1, dist) in similar_pairs {
        println!("  [{}] - [{}]  Distance: {}", name0, name1, dist);
    }
}
