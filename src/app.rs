//! Contains all the GUI code

use eframe::{
    egui::{self, Color32, Key, Stroke, StrokeKind, Vec2}, epaint::{CircleShape, RectShape},
};

use crate::{cullfile::Rating, image::ImageCollection, util::wrap, zoom_image_widget::ZoomImage};

// enum AppEvents {
//     GoToNextImage,
//     GoToPrevImage,
//     ResetZoom,
//     RateSelectedImage(Rating),
//     SaveCullfile,
// }

pub struct MyApp {
    images: ImageCollection,
    selected_image_index: usize,
    image_zoom_widget: ZoomImage,
}

impl MyApp {
    pub fn new(images: ImageCollection, ctx: &egui::Context) -> Self {
        let image_zoom_widget = ZoomImage::new(images[0].image_thumb.clone(), ctx);

        Self {
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
            let go_next = input.key_pressed(Key::ArrowRight) || input.key_pressed(Key::ArrowDown);
            let go_prev = input.key_pressed(Key::ArrowLeft) || input.key_pressed(Key::ArrowUp);

            let incr = match (go_next, go_prev) {
                (true, true) | (false, false) => 0, // Do nothing if both or neither are pressed
                (true, false) => 1,
                (false, true) => -1,
            };
            self.selected_image_index = wrap(
                self.selected_image_index as isize + incr,
                0,
                self.images.len() as isize,
            ) as usize;

            if input.key_pressed(Key::R) {
                self.image_zoom_widget.reset_zoom();
            }

            let rating = &mut self.images[self.selected_image_index].rating;
            if input.key_pressed(Key::Num5) { *rating = Rating::Five }
            else if input.key_pressed(Key::Num4) { *rating = Rating::Four }
            else if input.key_pressed(Key::Num3) { *rating = Rating::Three }
            else if input.key_pressed(Key::Num2) { *rating = Rating::Two }
            else if input.key_pressed(Key::Num1) { *rating = Rating::One }

            if input.key_pressed(Key::S) {
                println!("TODO: Handle saves");
            }
        });

        // Start loading the images ahead and behind the currently selected image
        let i = self.selected_image_index as isize;
        self.images.preload(i - 2..=i + 2);

        // Display the bottom status bar
        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "bottom panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
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

                    // Display the star rating of the image                    
                    for star_idx in 0..5 {
                        let (_, rect) = ui.allocate_space(Vec2::new(10.0, 10.0));
                        let mut circle = CircleShape::stroke(rect.center(), 5.0, Stroke::new(2.5, Color32::GRAY));
                        
                        // Fill the circles according to the image's rating
                        if star_idx < self.images[self.selected_image_index].rating as usize {
                            // ui.painter().add(CircleShape::filled(rect.center(), 5.0, Color32::YELLOW));
                            circle.fill = Color32::GRAY;
                        }
                        ui.painter().add(circle);
                    }

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
