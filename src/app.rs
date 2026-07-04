//! Contains all the GUI code

use std::sync::Arc;

use eframe::egui::{self, Image, Key, Sense, Vec2};

use crate::{cullfile::Cullfile, image::ImageWithMetadata, zoom_image_widget::ZoomImage};

pub struct MyApp {
    cullfile: Cullfile,
    images: Vec<ImageWithMetadata>,
    selected_image_index: usize,

    image_zoom_widget: ZoomImage,
}

impl MyApp {
    pub fn new(cullfile: Cullfile, images: Vec<ImageWithMetadata>) -> Self {
        let image_zoom_widget = ZoomImage::new(images[0].image.get_sized_texture());

        Self {
            cullfile,
            images,
            selected_image_index: 0,
            image_zoom_widget,
        }
    }

    fn selected_image(&self) -> &ImageWithMetadata {
        &self.images[self.selected_image_index]
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "bottom panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Image {} of {}",
                        self.selected_image_index + 1,
                        self.images.len()
                    ));
                    ui.label(format!(
                        "File name: {}",
                        self.selected_image()
                            .path_relative_to_cullfile.file_name().unwrap().to_str().unwrap()
                    ));
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.input(|input| {
                let go_next =
                    input.key_pressed(Key::ArrowRight) || input.key_pressed(Key::ArrowDown);
                let go_prev = input.key_pressed(Key::ArrowLeft) || input.key_pressed(Key::ArrowUp);

                match (go_next, go_prev) {
                    (true, true) | (false, false) => (), // Do nothing if both or neither are pressed
                    (true, false) => {
                        self.selected_image_index =
                            (self.selected_image_index + 1) % (self.images.len() - 1);
                    }
                    (false, true) => {
                        if self.selected_image_index == 0 {
                            self.selected_image_index = self.images.len() - 1;
                        } else {
                            self.selected_image_index -= 1;
                        }
                    }
                }
            });

            let image = &self.selected_image().image;
            self.image_zoom_widget
                .set_texture(image.get_sized_texture());
            ui.add(&mut self.image_zoom_widget);

            // let mut rect  = ui.allocate_exact_size(ui.available_size(), Sense::empty()).0;
            // egui::containers::Scene::new().show(ui, &mut rect, |ui| {
            //     ui.add(Image::new(self.selected_image().image.get_sized_texture()));
            // })
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
