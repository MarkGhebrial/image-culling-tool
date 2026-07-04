use std::path::Path;
use std::sync::Arc;
use std::time;
use std::{fs, path::PathBuf};

use image::ImageReader;
use rayon::prelude::*;

use crate::ImageWrapper;

pub struct ImageWithMetadata {
    /// The name of the file this image is stored in. Maybe I can instead store
    /// an `&std::fs::File`? Should make it easier to copy the image to a different
    /// directory.
    pub path_relative_to_cullfile: PathBuf,

    pub date_captured: time::SystemTime,

    pub image: Arc<ImageWrapper>,
}

/// Given a path, return all the images in that path
pub fn load_images(
    base_path: &Path,
    recursive: bool,
) -> Result<Vec<ImageWithMetadata>, std::io::Error> {
    if recursive {
        unimplemented!()
    }

    // For each item in the directory
    let dir: Vec<Result<std::fs::DirEntry, std::io::Error>> = fs::read_dir(base_path)?.collect();

    let progress_bar = indicatif::ProgressBar::new(dir.len() as u64);

    let images: Vec<ImageWithMetadata> = dir
        .into_par_iter()
        .filter(|_| {
            progress_bar.inc(1);
            true
        })
        .map(|entry_result| entry_result.unwrap())
        .filter(|entry| entry.path().is_file())
        .filter_map(|file| {
            let image = match ImageReader::open(file.path()).unwrap().decode() {
                Ok(image) => Some(image),
                Err(e) => match e {
                    image::ImageError::IoError(error) => Err(error).unwrap(),
                    _ => None
                }
            }?;

            Some(ImageWithMetadata {
                path_relative_to_cullfile: file.path(),
                date_captured: file.metadata().unwrap().created().unwrap(),
                image: Arc::new(ImageWrapper(image.to_rgb8())),
            })
        })
        .collect();
    
    Ok(images)
}
