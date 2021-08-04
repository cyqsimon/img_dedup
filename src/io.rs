//! This module contains functions that primarily deal with IO.
//!
//! This is not to say that other modules contain no IO code;
//! rather IO operations in other modules should be
//! secondary and/or supplementary to their functionality.

use crossbeam_channel::Sender;
use image::DynamicImage;
use regex::Regex;
use std::{
    fs::ReadDir,
    path::{Path, PathBuf},
};

/// This function tries to load all files in the opened directory
/// that matches the filter, tries to parse them into images,
/// and then send them via a channel.
///
/// This is a single-threaded operation.
///
/// If an error is encountered while loading or parsing an individual file,
/// it will be logged to console and skipped.
pub fn load_in(imgs_tx: Sender<(PathBuf, DynamicImage)>, opened_in_dir: ReadDir, in_filter: &Regex) {
    let selected_files: Vec<_> = opened_in_dir // short-circuit if error while opening directory
        // iter over io::Result<DirEntry>
        .map(|de_res| de_res.map(|de| de.path()))
        // iter over io::Result<PathBuf>
        .filter_map(|res| match res {
            Ok(path) => Some(path),
            Err(e) => {
                println!("Failed to open a file: {:?}", e);
                None
            }
        })
        // iter over PathBuf
        .filter(|path| match path.to_str() {
            Some(path_str) => in_filter.is_match(path_str),
            None => {
                println!("File path is not a valid utf-8 string: {:?}", path);
                false
            }
        })
        // iter over PathBuf (filtered)
        .collect();

    for path in selected_files.into_iter() {
        // read file and send to buffer
        match image::open(&path) {
            Ok(img) => {
                let send_res = imgs_tx.send((path.clone(), img)); // blocks if channel is full
                if let Err(e) = send_res {
                    println!("All image receivers hang up unexpectedly: {:?}", e);
                    println!("Image loading will stop now");
                    break;
                }
            }
            Err(e) => {
                println!("Failed to load {:?} as image: {:?}", &path, e);
            }
        };
    }
}

/// This function takes the filename from a path,
/// and tries to parse it into a UTF-8 string,
/// panicking if it's unable to do so.
///
/// This function is useful when the path's filename has
/// already been validated previously and does not need to be checked again.
pub fn get_filename_unchecked(path: &Path) -> &str {
    path.file_name()
        .expect("File \"..\" encountered unexpectedly.")
        .to_str()
        .expect("Bad file name (non-UTF8) encountered unexpectedly.")
}

/// This function checks that we are able to write a file
/// to a directory specified by its path.
///
/// A temporary file will be created in this path and
/// immediately deleted if there is no error.
pub fn test_write_to_dir(dir: &Path) -> std::io::Result<()> {
    use std::fs::create_dir_all as mkdir;
    use std::fs::read_dir;
    use std::fs::remove_file as rm;
    use std::fs::write;

    if dir.exists() {
        // check that dir can be opened
        let _ = read_dir(dir)?;
    } else {
        // try to mkdir
        mkdir(dir)?;
    }

    // find unused temp file name, just in case
    let tmp_file_name = (0..)
        .find_map(|n| {
            let mut test_path = dir.to_path_buf();
            test_path.push(format!("img-dedup-write-test-{}.tmp", n));

            (!test_path.exists()).then(|| test_path)
        })
        .unwrap(); // will find one eventually

    // try write to file
    write(&tmp_file_name, "This file is safe to delete.\n")?;

    // remove test file
    rm(&tmp_file_name)?;

    Ok(())
}
