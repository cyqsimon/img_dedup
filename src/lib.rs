use image::DynamicImage;
use regex::Regex;
use std::path::{Path, PathBuf};

pub fn load_in(in_dir: &Path, in_filter: &Regex) -> std::io::Result<Vec<std::io::Result<(PathBuf, DynamicImage)>>> {
    use std::{fs::read_dir, io::ErrorKind};

    let dir_children = read_dir(in_dir)?;
    Ok(dir_children
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
        .map(|p_res| match p_res {
            Ok(path) => match image::open(&path) {
                // if path can be opened as image, then return path-img pair
                Ok(img) => std::io::Result::Ok((path, img)),
                // else, produce IO error
                Err(e) => Err(std::io::Error::new(ErrorKind::InvalidData, e)),
            },
            Err(_) => Err(p_res.unwrap_err()),
        })
        // iter over io::Result<DynamicImage>
        .collect())
    // collected to Vec<io::Result<DynamicImage>> and wrap in Ok()
}
