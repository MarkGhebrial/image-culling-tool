use std::sync::Arc;

use eframe::{
    egui::{
        self, Color32, ImageData, Mesh, Pos2, Rect, Sense, TextureOptions, Vec2, Widget,
        load::SizedTexture,
    },
    epaint::ImageDelta,
};

use crate::util::rect_with_aspect_ratio;

pub struct ZoomImage {
    sized_texture: SizedTexture,

    /// The point in the image to zoom in to. From (0.0, 0.0) to (1.0, 1.0)
    zoom_translation: Vec2,

    /// The amount that the image is zoomed relative to its size when all the way
    /// zoomed out. Anything less than 1.0 is invalid
    zoom_scale: f32,
}

impl ZoomImage {
    pub fn new(image: Arc<impl ImageData>, ctx: &egui::Context) -> Self {
        let texture_id = ctx.tex_manager().write().alloc(
            "zoom image".to_owned(),
            image.clone(),
            TextureOptions::default(),
        );

        Self {
            sized_texture: SizedTexture::new(
                texture_id,
                [image.width() as f32, image.height() as f32],
            ),
            zoom_translation: Vec2::ZERO,
            zoom_scale: 1.0,
        }
    }

    pub fn set_image(&mut self, image: Arc<impl ImageData>, ctx: &egui::Context) {
        ctx.tex_manager().write().set(
            self.sized_texture.id,
            ImageDelta::full(image.clone(), TextureOptions::default()),
        );
        self.sized_texture.size = [image.width() as f32, image.height() as f32].into();
    }

    pub fn reset_zoom(&mut self) {
        self.zoom_translation = Vec2::ZERO;
        self.zoom_scale = 1.0;
    }
}

impl Widget for &mut ZoomImage {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        // The rect is the bounding box we have to work with. Maximally zoomed out,
        // two edges of the image align with two edges of the rect, and the other
        // two pairs of edges are equally distant from each other.
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::empty());

        let image_size = self.sized_texture.size;
        let img_aspect_ratio = image_size.x / image_size.y;

        let image_rect = rect_with_aspect_ratio(&rect, img_aspect_ratio)
            .translate(self.zoom_translation)
            .scale_from_center(self.zoom_scale);

        let image_rect_response = ui.allocate_rect(image_rect, Sense::HOVER | Sense::DRAG);

        // Adjust the image translation based on the drag amount
        let drag_amount = image_rect_response.drag_delta();

        // If the image is being hovered
        if let Some(hover_pos) = image_rect_response.hover_pos() {
            // Get access to the egui InputState and grab the things we need
            let (y_scroll, zoom_delta) = ui.input(|input| {
                // TODO: Use input.physical_pixel_size() to calculate the % zoom factor

                (input.smooth_scroll_delta.y, input.zoom_delta())
            });

            // Adjust the image zoom based on the scroll amount
            let mut new_zoom_scale = self.zoom_scale + (y_scroll * 0.05 * (self.zoom_scale));
            new_zoom_scale *= zoom_delta; // Handle touchscreen zooms
            if new_zoom_scale < 1.0 {
                new_zoom_scale = 1.0;
            }

            let new_image_rect = rect_with_aspect_ratio(&rect, img_aspect_ratio)
                .translate(self.zoom_translation)
                .scale_from_center(new_zoom_scale);

            // The cursor's location relative to the center of the original image rect, normalized from (-0.5, -0.5) to (0.5, 0.5)
            let relative_hover_position = (hover_pos - image_rect.center()) / image_rect.size();
            // The cursor's location relative to the center of the _new_ image rect, normalized. We want to transform the new image rect so that this
            let new_relative_hover_position =
                (hover_pos - new_image_rect.center()) / new_image_rect.size();

            // We need to translate the new image rect so that relative point 2 moves exactly on top of relative point one
            let zoom_correction_translation =
                (new_relative_hover_position - relative_hover_position) * new_image_rect.size();

            self.zoom_translation += zoom_correction_translation + drag_amount;
            self.zoom_scale = new_zoom_scale;
        }

        // Paint the image onto the image rect
        if ui.is_rect_visible(image_rect) {
            let mut mesh = Mesh::with_texture(self.sized_texture.id);
            mesh.add_rect_with_uv(
                image_rect,
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            ui.painter().add(mesh);
        }

        response
    }
}
