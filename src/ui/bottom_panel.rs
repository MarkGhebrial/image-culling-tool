use eframe::{
    egui::{self, Color32, Stroke, StrokeKind, Vec2},
    epaint::{CircleShape, RectShape},
};

use crate::{
    ui::{app_events::AppEvent, app_state::AppState, ui_component_trait::UiComponent},
    util,
};

pub struct BottomPanel; // Empty struct, since

impl UiComponent<AppState> for BottomPanel {
    fn display_ui(&self, state: &AppState, ui: &mut eframe::egui::Ui) -> Vec<AppEvent> {
        ui.horizontal(|ui| {
            let indicator_color = match state
                .images
                .cache
                .is_loaded(&state.selected_image().path_relative_to_cullfile)
            {
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
                let mut circle =
                    CircleShape::stroke(rect.center(), 5.0, Stroke::new(2.5, Color32::GRAY));

                // Fill the circles according to the image's rating
                if star_idx < state.selected_image().rating as usize {
                    circle.fill = Color32::GRAY;
                }
                ui.painter().add(circle);
            }

            ui.separator();

            // Display the current image index and the total number of images
            ui.label(format!(
                "{} of {}",
                state.selected_image_index + 1,
                state.images.len()
            ));

            ui.separator();

            // Display the file name of the current image
            ui.label(format!(
                "{}",
                state
                    .selected_image()
                    .path_relative_to_cullfile
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            ));

            // Display the stuff on the right side of the status bar
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("Test");
                ui.separator();
                ui.label("Test2");
                ui.separator();
                ui.add(
                    egui::ProgressBar::new(0.75)
                        .desired_width(util::min(ui.available_width(), 200.0))
                        .desired_height(5.0),
                );
            });
        });

        Vec::new()
    }

    fn react_to_events(&mut self, state: &mut AppState, _events: &Vec<AppEvent>) {}
}
