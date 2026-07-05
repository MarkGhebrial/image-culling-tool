use std::sync::Arc;

use eframe::{
    egui::{
        self, Color32, ImageData, Mesh, Pos2, Rect, Sense, TextureId, TextureOptions, Vec2, Widget,
    },
    epaint::ImageDelta,
};

pub struct ZoomImage {
    texture_id: TextureId,
    // The width and height of the image in pixels
    image: Arc<dyn ImageData>,

    /// The point in the image to zoom in to. From (0.0, 0.0) to (1.0, 1.0)
    zoom_translation: Vec2,

    /// The amount that the image is zoomed in. 0.0 means "fit the image to the\
    /// bounding rect" and 1.0 means "idk bro, just zoom the image in all the way"
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
            texture_id,
            image,
            zoom_translation: Vec2::new(0.0, 0.0),
            zoom_scale: 1.0,
        }
    }

    pub fn set_image(&mut self, image: Arc<impl ImageData>, ctx: &egui::Context) {
        ctx.tex_manager().write().set(
            self.texture_id,
            ImageDelta::full(image.clone(), TextureOptions::default()),
        );
        self.image = image;
    }
}

impl Widget for &mut ZoomImage {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        // The rect is the bounding box we have to work with. Maximally zoomed out,
        // two edges of the image align with two edges of the rect, and the other
        // two pairs of edges are equally distant from each other.
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::all());

        // width/height
        // Larger aspect ratio -> wider, shorter
        // Smaller aspect ratio -> taller, skinnier
        let rect_aspect_ratio = rect.aspect_ratio();
        let img_aspect_ratio = self.image.size()[0] as f32 / self.image.size()[1] as f32;

        // If the widget rect is wider than the image's rect, their heights should be the same
        let mut image_rect = if rect_aspect_ratio >= img_aspect_ratio {
            Rect::from_center_size(
                rect.center(),
                Vec2 {
                    x: rect.height() * img_aspect_ratio,
                    y: rect.height(),
                },
            )
        }
        // Else, their widths should be the same
        else {
            Rect::from_center_size(
                rect.center(),
                Vec2 {
                    x: rect.width(),
                    y: rect.width() / img_aspect_ratio,
                },
            )
        };

        // Translate and scale the image
        image_rect = image_rect
            .translate(self.zoom_translation)
            .scale_from_center(self.zoom_scale);
        let image_rect_response = ui.allocate_rect(image_rect, Sense::all());

        // If the image is being hovered
        if let Some(hover_pos) = image_rect_response.hover_pos() {
            // Get access to the egui InputState
            ui.input_mut(|input| {
                // TODO: Use this to calculate the % zoom factor
                input.physical_pixel_size();

                let drag_amount =
                    if input.pointer.is_decidedly_dragging() && input.pointer.primary_down() {
                        let ptr = &input.pointer;
                        // if let (Some(origin), Some(current_position)) = (ptr.press_origin(), ptr.interact_pos()) {
                        //     origin - current_position
                        // } else {
                        //     Vec2::ZERO
                        // }
                        ptr.delta()
                        // let o = input.pointer.press_origin();
                        // let c = input.pointer.interact_pos();.map(|pos| pos - input.pointer.press_origin().ma);

                        // let drag_amount = c - o;

                        // println!("Pointer is dragging from {:?} to {:?}", o, c);
                    } else {
                        Vec2::ZERO
                    };

                let Vec2 {
                    y: y_scroll,
                    x: x_scroll,
                } = input.smooth_scroll_delta;
                // Reset the scroll amount so that any scroll containers don't also scroll in this frame
                input.raw_scroll_delta = Vec2::ZERO;

                println!("Touch zoom {}", input.zoom_delta());
                println!("Scroll: ({}, {})", x_scroll, y_scroll);

                // let normalized_hover_pos = hover_pos.to_vec2() / rect.size();

                // Adjust the image translation based on the drag amount
                self.zoom_translation += drag_amount;

                // Adjust the image zoom based on the scroll amount
                self.zoom_scale += y_scroll * 0.05 * (self.zoom_scale);
                self.zoom_scale *= input.zoom_delta(); // Handle touchscreen zooms
                if self.zoom_scale < 1.0 {
                    self.zoom_scale = 1.0;
                }
            });
        }

        if ui.is_rect_visible(rect) {
            let mut mesh = Mesh::with_texture(self.texture_id);
            mesh.add_rect_with_uv(
                image_rect, //rect,
                Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
            ui.painter().add(mesh);
        }

        response
    }
}
