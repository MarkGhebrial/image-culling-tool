//! Contains all the GUI code

use eframe::{
    egui::{self, Color32, Key, ProgressBar, Stroke, StrokeKind, Vec2},
    epaint::{CircleShape, RectShape},
};

use crate::{
    cullfile::Rating,
    image::ImageCollection,
    ui::{app_state::AppState, components, ui_component_trait::UiComponent},
    util::{self, wrap},
    zoom_image_widget::ZoomImage,
};

pub struct MyApp {
    state: AppState,

    bottom_panel: components::BottomPanel,
    // images: ImageCollection,
    // selected_image_index: usize,
    image_zoom_widget: ZoomImage,
}

impl MyApp {
    pub fn new(images: ImageCollection, ctx: &egui::Context) -> Self {
        let image_zoom_widget = ZoomImage::new(images[0].image_thumb.clone(), ctx);

        Self {
            state: AppState {
                images,
                selected_image_index: 0,
            },
            bottom_panel: components::BottomPanel,
            image_zoom_widget,
        }
    }

    fn save(&mut self) {
        self.state.images.save_cullfile();
    }
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
            self.state.selected_image_index = wrap(
                self.state.selected_image_index as isize + incr,
                0,
                self.state.images.len() as isize,
            ) as usize;

            if input.key_pressed(Key::R) {
                self.image_zoom_widget.reset_zoom();
            }

            let rating = &mut self.state.selected_image_mut().rating;
            if input.key_pressed(Key::Num5) {
                *rating = Rating::Five
            } else if input.key_pressed(Key::Num4) {
                *rating = Rating::Four
            } else if input.key_pressed(Key::Num3) {
                *rating = Rating::Three
            } else if input.key_pressed(Key::Num2) {
                *rating = Rating::Two
            } else if input.key_pressed(Key::Num1) {
                *rating = Rating::One
            }

            if input.key_pressed(Key::S) {
                self.save();
            }
        });

        // Start loading the images ahead and behind the currently selected image
        let i = self.state.selected_image_index as isize;
        self.state.images.preload(i - 2..=i + 2);

        let response = egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "bottom panel")
            .resizable(false)
            .show(ctx, |ui| self.bottom_panel.display_ui(&self.state, ui));

        response.inner;

        // Draw the central panel. The part of the screen where the selected image is displayed
        egui::CentralPanel::default().show(ctx, |ui| {
            let image = self
                .state
                .images
                .get_full_resolution_image(self.state.selected_image_index);
            self.image_zoom_widget.set_image(image, ctx);
            ui.add(&mut self.image_zoom_widget);
        });
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        println!("eframe autosave");
        self.save();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save();
    }

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
