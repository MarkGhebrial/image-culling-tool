//! Contains all the GUI code

use std::sync::Arc;

use eframe::egui::{self, Image, Key, Vec2};

use crate::{cullfile::Cullfile, image::ImageWithMetadata};

pub struct MyApp {
    cullfile: Cullfile,
    images: Vec<ImageWithMetadata>,
    selected_image_index: usize,
}

impl MyApp {
    pub fn new(cullfile: Cullfile, images: Vec<ImageWithMetadata>) -> Self {
        Self { cullfile, images, selected_image_index: 0 }
    }

    fn selected_image(&self) -> &ImageWithMetadata {
        &self.images[self.selected_image_index]
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.input(|input| {
                let go_next = input.key_pressed(Key::ArrowRight) || input.key_pressed(Key::ArrowDown);
                let go_prev = input.key_pressed(Key::ArrowLeft) || input.key_pressed(Key::ArrowUp);

                match (go_next, go_prev) {
                    (true, true) | (false, false) => (), // Do nothing if both or neither are pressed
                    (true, false) => {
                        self.selected_image_index = (self.selected_image_index + 1) % (self.images.len() - 1);
                    },
                    (false, true) => {
                        if self.selected_image_index == 0 {
                            self.selected_image_index = self.images.len() - 1;
                        }
                        else {
                            self.selected_image_index -= 1;
                        }
                    },
                }
            });

            let image = &self.selected_image().image;

            let im_widget = Image::new(image.get_sized_texture()).fit_to_fraction(Vec2::new(1.0, 1.0));
            println!("Widget size: {:?}", im_widget.size());
            ui.add(im_widget);

            ui.label("Below the image :)");
        });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.cullfile.save();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).to_normalized_gamma_f32()

        // _visuals.window_fill() would also be a natural choice
    }
}
