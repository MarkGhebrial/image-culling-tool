use std::sync::Arc;

use eframe::{
    egui::{self, ImageData, ImageSource, TextureId, TextureOptions, load::SizedTexture},
    epaint::TextureManager,
};

pub struct TexBox<T> {
    image: Arc<T>,
    texture_manager: Arc<egui::mutex::RwLock<TextureManager>>,
    texture_id: TextureId,
}
impl<T> TexBox<T>
where
    T: ImageData,
{
    pub fn new(image: Arc<T>, texture_manager: Arc<egui::mutex::RwLock<TextureManager>>) -> Self {
        let texture_id = texture_manager.write().alloc(
            "texbox texture".to_owned(),
            image.clone(),
            TextureOptions::default(),
        );

        Self {
            image,
            texture_manager,
            texture_id,
        }
    }

    pub fn get_image(&self) -> Arc<T> {
        self.image.clone()
    }

    pub fn get_texture_id(&self) -> TextureId {
        self.texture_id.clone()
    }

    pub fn get_sized_texture(&self) -> SizedTexture {
        SizedTexture {
            id: self.texture_id.clone(),
            size: [self.image.width() as f32, self.image.height() as f32].into(),
        }
    }
}
impl<T> Drop for TexBox<T> {
    fn drop(&mut self) {
        // If there's only one reference to the image remaining
        if Arc::strong_count(&self.image) == 1 {
            self.texture_manager.write().free(self.texture_id.clone());
        }
    }
}

impl<'a, T> Into<ImageSource<'a>> for &'a TexBox<T>
where
    T: ImageData,
{
    fn into(self) -> ImageSource<'a> {
        ImageSource::Texture(self.get_sized_texture())
    }
}
