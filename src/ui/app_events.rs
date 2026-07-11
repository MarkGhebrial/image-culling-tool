use crate::cullfile::Rating;

/// Enumeration of events that mutate application state and must happen in response to user input
pub enum AppEvent {
    GoToNextImage,
    GoToPrevImage,
    ResetZoom,
    RateSelectedImage(Rating),
    SaveCullfile,
}
