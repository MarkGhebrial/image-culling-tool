use std::path::Path;
use std::sync::Arc;
use std::time;
use std::{fs, path::PathBuf};

use eframe::egui;
use eframe::epaint::TextureManager;
use image::ImageReader;
use indicatif::ProgressIterator;

use crate::{ImageWrapper, TexBox};

pub struct ImageWithMetadata {
    /// The name of the file this image is stored in. Maybe I can instead store
    /// an `&std::fs::File`? Should make it easier to copy the image to a different
    /// directory.
    pub path_relative_to_cullfile: PathBuf,

    pub date_captured: time::SystemTime,

    pub image: TexBox<ImageWrapper>,
}

/// Given a path, return all the images in that path
pub fn load_images(
    base_path: &Path,
    recursive: bool,
    texture_manager: Arc<egui::mutex::RwLock<TextureManager>>,
) -> Result<Vec<ImageWithMetadata>, std::io::Error> {
    if recursive {
        unimplemented!()
    }

    let mut images = Vec::new();

    // let pb = indicatif::ProgressBar::new(3000);

    // For each item in the directory
    let dir: Vec<Result<std::fs::DirEntry, std::io::Error>> = fs::read_dir(base_path)?.collect();
    for entry in dir.into_iter().progress() {
        let entry: std::fs::DirEntry = entry?;
        // If the item is a file
        if entry.path().is_file() {
            // Try to load the file as an image
            let image = match ImageReader::open(entry.path())?.decode() {
                Ok(image) => image,
                // If the file isn't an image, ignore it
                Err(e) => match e {
                    image::ImageError::IoError(error) => return Err(error),
                    _ => continue, // Ignore "decoding", "encoding", "parameter", "limits", and "unsupported" errors
                },
            };

            images.push(ImageWithMetadata {
                path_relative_to_cullfile: entry.path().to_owned(),
                date_captured: entry.metadata()?.created()?,
                image: TexBox::new(
                    Arc::new(ImageWrapper(image.to_rgb8())),
                    texture_manager.clone(),
                ),
            });
        }
    }

    Ok(images)
}
