use image::{DynamicImage, ImageResult};
use img_hash::{HasherConfig, ImageHash};
use regex::Regex;
use std::{fs::read_dir, io::ErrorKind, path::Path};

pub fn load_in(in_dir: &Path, in_filter: Regex) -> std::io::Result<Vec<ImageResult<DynamicImage>>> {
    read_dir(in_dir)?
        // iter over io::Result<DirEntry>
        .map(|de_res| de_res.map(|de| de.path()))
        // iter over io::Result<PathBuf>
        .filter_map(|p_res| match &p_res {
            Ok(p) => match p.to_str() {
                // if path can be parsed into str, then filter
                Some(ps) => in_filter.is_match(ps).then(|| p_res),
                // if path cannot be represented by str, then produce IO error
                None => Some(Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "File path is not valid utf-8 string",
                ))),
            },
            Err(_) => Some(p_res),
        })
        // filtered with regex
        .map(|p_res| p_res.map(|path| image::open(&path)))
        .collect()
}

pub fn gen_hashes(in_imgs: Vec<DynamicImage>) -> Vec<(DynamicImage, ImageHash)> {
    let hasher = HasherConfig::new().to_hasher();
    in_imgs
        .into_iter()
        .map(|img| {
            let h = hasher.hash_image(&img);
            (img, h)
        })
        .collect()
}
