//! Contains all the GUI code

use std::sync::Arc;

use eframe::egui::{self, Image, ImageData, ImageSource};

use crate::{cullfile::Cullfile, image::ImageWithMetadata};

pub struct MyApp {
    cullfile: Cullfile,
    images: Vec<ImageWithMetadata>,
}

impl MyApp {
    pub fn new(cullfile: Cullfile, images: Vec<ImageWithMetadata>) -> Self {
        Self { cullfile, images }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Cull tool");

            let image = &self.images[0].image;

            // ui.image("https://ghebrial.net/images/san-clemente-beach/41083013750-R1-065-31.jpg");

            println!("Image size: {:?}", image.size());
            // image.size();

            let im_widget = Image::from_image_data(self.images[0].image.clone());
            println!("Widget size: {:?}", im_widget.size());
            ui.add(im_widget);

            ui.label("Below the image :)");

            println!("{}", Arc::strong_count(&self.images[0].image));
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
