use crossbeam_channel::Sender;
use image::DynamicImage;
use regex::Regex;
use std::{
    fs::ReadDir,
    path::{Path, PathBuf},
};

pub fn load_in(img_send: Sender<(PathBuf, DynamicImage)>, opened_in_dir: ReadDir, in_filter: &Regex) {
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
                let send_res = img_send.send((path.clone(), img)); // blocks if channel is full
                if let Err(e) = send_res {
                    println!("All img_queue receivers hang up unexpectedly: {:?}", e);
                    println!("Image loading will stop now");
                    break;
                }
            }
            Err(e) => {
                println!("Failed to load {:?} as image: {:?}", &path, e);
            }
        };
    }

    // cleanup: we might have to manually hang up sender here
    //drop(img_send);
}

pub fn get_filename_unchecked(path: &Path) -> &str {
    path.file_name()
        .expect("File \"..\" encountered unexpectedly.")
        .to_str()
        .expect("Bad file name (non-UTF8) encountered unexpectedly.")
}
