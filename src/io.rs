use crossbeam_channel::Sender;
use image::DynamicImage;
use regex::Regex;
use std::{
    fs::ReadDir,
    path::{Path, PathBuf},
};

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

pub fn get_filename_unchecked(path: &Path) -> &str {
    path.file_name()
        .expect("File \"..\" encountered unexpectedly.")
        .to_str()
        .expect("Bad file name (non-UTF8) encountered unexpectedly.")
}

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

    // find unused temp file name
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
