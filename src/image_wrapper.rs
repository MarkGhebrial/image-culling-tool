use std::ops::Deref;

use eframe::epaint;
use image::{EncodableLayout, RgbImage};

#[derive(Debug)]
pub struct ImageWrapper(pub RgbImage);

impl epaint::ImageData for ImageWrapper {
    fn size(&self) -> [usize; 2] {
        [
            self.0.dimensions().0 as usize,
            self.0.dimensions().1 as usize,
        ]
    }

    fn width(&self) -> usize {
        self.size()[0]
    }

    fn height(&self) -> usize {
        self.size()[1]
    }

    fn pixel_type(&self) -> epaint::image::PixelType {
        epaint::image::PixelType::Rgb
    }

    fn data(&self) -> &[u8] {
        &self.0.as_bytes()
    }
}

impl Deref for ImageWrapper {
    type Target = RgbImage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}