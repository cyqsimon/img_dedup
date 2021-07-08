use image::DynamicImage;
use regex::Regex;
use std::{
    path::{Path, PathBuf},
    sync::RwLock,
};

pub fn load_in(
    img_queue: &RwLock<Vec<(PathBuf, DynamicImage)>>,
    in_dir: &Path,
    in_filter: &Regex,
) -> std::io::Result<()> {
    use std::fs::read_dir;

    // Number of files load at a time before trying a soft write-lock and push
    // If cannot lock, then keep loading next batch
    let load_batch_size = num_cpus::get();
    // Maximum number of files allowed before requesting a hard write-lock and push
    // Blocks until we can push into img_queue
    let load_buffer_max = 128;

    let selected_files: Vec<_> = read_dir(in_dir)? // short-circuit if error while opening directory
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

    // this buffer stores the imgs if the batches accumulate
    let mut img_store_buffer = vec![];

    for batch in selected_files.chunks(load_batch_size) {
        // load the batch of imgs into temp vec
        let mut batch_img_store_tmp: Vec<_> = batch
            .into_iter()
            .filter_map(|path| match image::open(path) {
                Ok(img) => Some((path.clone(), img)),
                Err(e) => {
                    println!("Failed to load {:?} as image: {:?}", path, e);
                    None
                }
            })
            // iter over (PathBuf, DynamicImage)
            .collect();

        // move batch into buffer
        img_store_buffer.append(&mut batch_img_store_tmp);

        // attempt soft write-lock
        match img_queue.try_write() {
            Ok(mut vec) => vec.append(&mut img_store_buffer),
            Err(_) => {
                // if failed, check whether we should hard write-lock
                if img_store_buffer.len() > load_buffer_max {
                    img_queue
                        .write()
                        // block here until lock acquired
                        .expect("A thread panicked while holding exclusive write-lock to img queue")
                        .append(&mut img_store_buffer);
                }
            }
        }
    }

    // drain buffer if necessary
    if !img_store_buffer.is_empty() {
        img_queue
            .write()
            // block here until lock acquired
            .expect("A thread panicked while holding exclusive write-lock to img queue")
            .append(&mut img_store_buffer);
    }

    Ok(())
}

pub fn get_filename_unchecked(path: &Path) -> &str {
    path.file_name()
        .expect("File \"..\" encountered unexpectedly.")
        .to_str()
        .expect("Bad file name (non-UTF8) encountered unexpectedly.")
}
