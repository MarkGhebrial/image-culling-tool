use eframe::egui;

use crate::ui::app_events::AppEvent;

/// Some part of the user interface
pub trait UiComponent<T> {
    /// Display the component. This function must not mutate the component's state; that should happen
    /// in `react_to_events`
    #[must_use = "App events must be handled"]
    fn display_ui(&self, state: &T, ui: &mut egui::Ui) -> Vec<AppEvent>;

    fn react_to_events(&mut self, state: &mut T, events: &Vec<AppEvent>);
}
