use std::{ops::Deref, sync::Arc};

use eframe::{
    egui::{ImageData, TextureId, TextureOptions, mutex::RwLock},
    epaint::TextureManager,
};

pub struct AllocatedTexture<T>
where
    T: ImageData,
{
    texture_manager: Arc<RwLock<TextureManager>>,
    texture_id: TextureId,
    image: T,
}

impl<T> AllocatedTexture<T>
where
    T: ImageData + Clone,
{
    pub fn new(texture_manager: Arc<RwLock<TextureManager>>, image: T) -> Self {
        let texture_id =
            texture_manager
                .write()
                .alloc(String::new(), image.clone(), TextureOptions::default());

        Self {
            texture_manager,
            texture_id,
            image,
        }
    }
}

impl<T> Drop for AllocatedTexture<T>
where
    T: ImageData,
{
    fn drop(&mut self) {
        self.texture_manager.write().free(self.texture_id);
    }
}

impl<T> Deref for AllocatedTexture<T>
where
    T: ImageData,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}
