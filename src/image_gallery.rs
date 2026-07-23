use eframe::{
    egui::{self, Color32, ImageData, Mesh, Pos2, Rect, Sense, Vec2},
    epaint::RectShape,
};

use crate::{
    image::ImageCollection,
    util::{max, rect_with_aspect_ratio},
};

pub struct ImageGallery {
    min_image_width: f32,

    scroll_position: f32,
}

impl ImageGallery {
    pub fn new(min_image_width: f32) -> Self {
        ImageGallery {
            min_image_width,
            scroll_position: 0.0,
        }
    }

    pub fn show(
        &mut self,
        images: &ImageCollection,
        selected_image_index: &mut usize,
        ui: &mut egui::Ui,
    ) {
        let scroll_output = egui::ScrollArea::vertical().show(ui, |ui| {
            let available_width = ui.available_width();

            let images_per_row: usize = max(1, (available_width / self.min_image_width) as usize);
            let image_width = available_width / images_per_row as f32;

            let mut current_col: usize = 0;
            let mut current_row: usize = 0;
            for (i, image) in images.iter().enumerate() {
                let rect = Rect::from_min_max(
                    Pos2::new(
                        image_width * current_col as f32,
                        image_width * current_row as f32,
                    ),
                    Pos2::new(
                        image_width * (current_col + 1) as f32,
                        image_width * (current_row + 1) as f32,
                    ),
                )
                .translate(Vec2::new(0.0, -self.scroll_position));

                let response = ui.allocate_rect(rect, Sense::hover() | Sense::click());

                if ui.is_rect_visible(rect) {
                    let corner_radius: f32 = 4.0;

                    // Display a blue background for the currently selected image
                    if i == *selected_image_index {
                        let bg_color = ui.style().visuals.selection.bg_fill;
                        ui.painter()
                            .add(RectShape::filled(rect, corner_radius, bg_color));
                    }
                    // Display a gray background for the currently hovered image
                    else if response.hovered() {
                        ui.painter()
                            .add(RectShape::filled(rect, corner_radius, Color32::GRAY));
                    }

                    if response.clicked() {
                        *selected_image_index = i;
                    }

                    let img_aspect_ratio =
                        image.image_thumb.size()[0] as f32 / image.image_thumb.size()[1] as f32;

                    // Display the image thumbnail
                    let mut mesh = Mesh::with_texture(image.thumb_texture);
                    mesh.add_rect_with_uv(
                        rect_with_aspect_ratio(&rect.shrink(2.0), img_aspect_ratio),
                        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                    ui.painter().add(mesh);
                }

                current_col += 1;
                if current_col >= images_per_row {
                    current_col = 0;
                    current_row += 1;
                }
            }
        });

        self.scroll_position = scroll_output.state.offset.y
    }
}
