use crate::image::{ImageCollection, ImageWithMetadata};

/// The extra state passed around to ui components
pub struct AppState {
    pub images: ImageCollection,
    pub selected_image_index: usize,
}

impl AppState {
    pub fn selected_image(&self) -> &ImageWithMetadata {
        &self.images[self.selected_image_index]
    }

    pub fn selected_image_mut(&mut self) -> &mut ImageWithMetadata {
        &mut self.images[self.selected_image_index]
    }
}
