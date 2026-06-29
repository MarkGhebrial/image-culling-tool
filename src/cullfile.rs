use std::{collections::HashMap, path::Path};

#[repr(usize)]
#[derive(Clone, Copy)]
pub enum Rating {
    Unrated = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
}

impl Rating {
    pub fn from_int(value: usize) -> Option<Self> {
        match value {
            0 => Some(Self::Unrated),
            1 => Some(Self::One),
            2 => Some(Self::Two),
            3 => Some(Self::Three),
            4 => Some(Self::Four),
            5 => Some(Self::Five),
            _ => None,
        }
    }
}

pub struct Cullfile {
    // Map an image file name to its rating
    ratings: HashMap<String, Rating>,
}

impl Cullfile {
    /// Load the contents of the current directory's cullfile.
    pub fn load(path: &Path) -> Self {
        let mut ratings = HashMap::new();

        // Get the current working directory
        let Ok(file) = std::fs::read_to_string(path.join(".cullfile")) else {
            return Self { ratings };
        };

        for line in file.lines() {
            let (rating, file_path) = line
                .split_once(" ")
                .expect("Error parsing cullfile: couldn't split line");

            let rating: Rating = Rating::from_int(
                rating
                    .parse()
                    .expect("Error while parsing cullfile: malformed rating"),
            )
            .expect(
                "Error parsing cullfile: rating not in range (must be between 0 and 5, inclusive)",
            );

            ratings.insert(file_path.to_owned(), rating);
        }

        Self { ratings }
    }

    pub fn save(&self) {
        // Serialize self and save to ".cullfile" in the current working directory, overwriting it if it already exists
        todo!()
    }

    pub fn set_rating(&mut self, image_file_name: &str, rating: Rating) {
        self.ratings.insert(image_file_name.to_owned(), rating);
    }

    pub fn get_rating(&self, image_file_name: &str) -> Rating {
        self.ratings
            .get(image_file_name)
            .map(|r| r.clone())
            .unwrap_or(Rating::Unrated)
    }
}
