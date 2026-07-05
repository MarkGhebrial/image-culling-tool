use std::path::Path;
use std::sync::Arc;
use std::time;
use std::{fs, path::PathBuf};

use image::ImageReader;
use image::{DynamicImage, ImageDecoder, ImageError};
use rayon::prelude::*;

use crate::image_wrapper::ImageWrapper;

pub struct ImageWithMetadata {
    /// The name of the file this image is stored in. Maybe I can instead store
    /// an `&std::fs::File`? Should make it easier to copy the image to a different
    /// directory.
    pub path_relative_to_cullfile: PathBuf,

    pub date_captured: time::SystemTime,

    pub image_thumb: Arc<ImageWrapper>,
}

/// Given a path, return all the images in that path
pub fn load_images(
    base_path: &Path,
    recursive: bool,
    // The size of the image thumbnails to generate, in pixels. For images that are
    // not square, this specifies the length of their long edge
    thumb_size: u32,
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
            Some(ImageWithMetadata {
                path_relative_to_cullfile: file.path(),
                date_captured: file.metadata().unwrap().created().unwrap(),
                image_thumb: match load_image_from_file(file.path(), thumb_size) {
                    Ok(image) => image,
                    Err(ImageError::IoError(e)) => return None, // TODO: Throw an error here
                    _ => return None,
                },
            })
        })
        .collect();

    Ok(images)
}

/// Load an image from the given file path. If `thumb_size != 0`, then create a lower
/// resolution version of the image.
fn load_image_from_file(
    path: impl AsRef<Path>,
    thumb_size: u32,
) -> Result<Arc<ImageWrapper>, image::ImageError> {
    let mut image_decoder = ImageReader::open(path)?.into_decoder()?;
    let orientation = image_decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);

    let mut image = DynamicImage::from_decoder(image_decoder)?;
    image.apply_orientation(orientation);

    if thumb_size != 0 {
        image = image.thumbnail(thumb_size, thumb_size);
    }

    Ok(Arc::new(ImageWrapper(image.to_rgb8())))
}

pub struct ImageLoader;
impl crate::async_runtime::AsyncLruCacheLoader for ImageLoader {
    type Key = PathBuf;
    type Value = Arc<ImageWrapper>;

    async fn load(key: Self::Key) -> Self::Value {
        // Use the `blocking` crate to offload the io operation to a different thread, and await the completion of that thread
        blocking::unblock(|| load_image_from_file(key, 0))
            .await
            .unwrap()
    }
}
