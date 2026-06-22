
pub struct ImageWithMetadata {
    saved: bool,

    date_captured: chrono::NaiveDateTime,

    /// Ranges from 0 to 5. 0 indicates no rating, 1-5 indicates the number of stars rated.
    rating: u8,
}

impl ImageWithMetadata {
    pub fn load_from_disk(path: &str) -> Self {
        todo!()
    }

    pub fn set_rating(&mut self, new_rating: u8) {
        self.saved = false;

        if new_rating > 5 {
            self.rating = 5;
        } else {
            self.rating = new_rating;
        }
    }

    // pub fn 
}