mod clap_def;
mod cli_helper;
mod compute;
mod io;
mod sub_cmds;
mod sub_ops;

use crossbeam_channel::bounded;
use regex::Regex;
use std::{fs::read_dir, path::Path, process::exit, thread, time::Duration};

use crate::{
    clap_def::build_app,
    io::load_in,
    sub_cmds::{hash_once, move_duplicates, scan_duplicates},
};

fn main() {
    let clap_matches = build_app().get_matches();

    // create single-producer, multiple-consumer channel
    let (imgs_tx, imgs_rx) = bounded(128);

    // get input options
    let in_dir = clap_matches.value_of("input_dir").unwrap(); // arg is required
    let in_filter_regex = Regex::new(
        clap_matches.value_of("input_filter").unwrap(), // default provided by clap
    )
    .unwrap(); // regex validated by clap

    // get concurrency options
    let concurrency = clap_matches
        .value_of("concurrency")
        .unwrap() // default provided by clap
        .parse::<usize>()
        .unwrap(); // usize parse validated by clap

    // opening imgs_dir outside of thread makes for easier code logic
    let opened_imgs_dir = read_dir(Path::new(in_dir)).unwrap_or_else(|e| {
        println!("Failed to open the input directory: {:?}", e);
        exit(1);
    });

    // start imgs loading (single producer)
    println!(
        "Loading files in [{}] with regex filter [/{}/]...",
        in_dir,
        in_filter_regex.as_str()
    );
    thread::spawn(move || {
        load_in(imgs_tx, opened_imgs_dir, &in_filter_regex);
    });

    // spawn image loader monitor daemon
    let imgs_rx_monitor = imgs_rx.clone();
    let (monitor_kill_tx, monitor_kill_rx) = bounded(0);
    thread::spawn(move || 'thread: loop {
        let queue_len = imgs_rx_monitor.len();
        if queue_len > 0 {
            println!(
                "IO loading images faster than we can hash; currently {} in queue",
                queue_len
            );
        }
        // sleep for 5s total, but check for termination every 100ms
        for _ in 0..50 {
            if let Ok(_) = monitor_kill_rx.try_recv() {
                break 'thread;
            }
            thread::sleep(Duration::from_millis(100));
        }
    });

    // log concurrency info
    println!("Using up to {} threads", concurrency);

    // dispatch task to subcmds
    match clap_matches.subcommand() {
        ("hash", Some(sub_matches)) => {
            let _ = hash_once(imgs_rx, concurrency, sub_matches);
        }
        ("scan-duplicates", Some(sub_matches)) => {
            let _ = scan_duplicates(imgs_rx, concurrency, sub_matches);
        }
        ("move-duplicates", Some(sub_matches)) => {
            move_duplicates(imgs_rx, concurrency, sub_matches);
        }
        _ => unreachable!("Cases should always cover all defined subcmds"),
    };

    // stop monitoring daemon
    monitor_kill_tx
        .send(())
        .expect("Image loader monitor daemon failed unexpectedly");
}
