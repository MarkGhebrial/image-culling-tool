//! Contains all the GUI code

use eframe::{
    egui::{self, Color32, Key, Stroke, StrokeKind, Vec2},
    epaint::RectShape,
};

use crate::{
    cullfile::Cullfile, image::ImageCollection, zoom_image_widget::ZoomImage,
};

// enum AppEvents {
//     GoToNextImage,
//     GoToPrevImage,
//     ResetZoom,
//     RateSelectedImage(Rating),
//     SaveCullfile,
// }

pub struct MyApp {
    cullfile: Cullfile,

    images: ImageCollection,
    selected_image_index: usize,
    image_zoom_widget: ZoomImage,
}

impl MyApp {
    pub fn new(cullfile: Cullfile, images: ImageCollection, ctx: &egui::Context) -> Self {
        let image_zoom_widget = ZoomImage::new(images[0].image_thumb.clone(), ctx);

        Self {
            cullfile,
            images,
            selected_image_index: 0,
            image_zoom_widget,
        }
    }

    // fn selected_image(&self) -> &ImageWithMetadata {
    //     &self.images[self.selected_image_index]
    // }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Read keypresses and input events
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
            
        // Display the bottom status bar
        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "bottom panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    println!("`is_loaded` key: {:?}", self.images[self.selected_image_index].path_relative_to_cullfile);

                    let indicator_color = match self.images.cache.is_loaded(
                        &self.images[self.selected_image_index].path_relative_to_cullfile,
                    ) {
                        true => Color32::GREEN,
                        false => Color32::RED,
                    };

                    // Draw the indicator in the bottom left corner
                    let (_, mut rect) = ui.allocate_space(Vec2::new(0.0, 0.0));
                    // Expand the rectangle to take up the top and left margins
                    rect = rect
                        .expand2(Vec2::new(0.0, ui.available_height() - rect.height()))
                        .with_min_x(0.0);
                    ui.painter().add(RectShape::new(
                        rect,
                        2.0,
                        indicator_color,
                        Stroke::NONE,
                        StrokeKind::Inside,
                    ));

                    ui.label(format!(
                        "{}",
                        self.cullfile.get_rating(
                            &self.images[self.selected_image_index].path_relative_to_cullfile
                        )
                    ));

                    ui.separator();

                    ui.label(format!(
                        "{} of {}",
                        self.selected_image_index + 1,
                        self.images.len()
                    ));

                    ui.separator();

                    ui.label(format!(
                        "{}",
                        self.images[self.selected_image_index]
                            .path_relative_to_cullfile
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                    ));
                });
            });

        // Draw the central panel. The part of the screen where the selected image is displayed
        egui::CentralPanel::default().show(ctx, |ui| {
            let image = self
                .images
                .get_full_resolution_image(self.selected_image_index);
            self.image_zoom_widget.set_image(image, ctx);
            ui.add(&mut self.image_zoom_widget);
        });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        println!("eframe autosave")
        // TODO: Save cullfile
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
