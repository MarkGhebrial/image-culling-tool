use std::io::Write;
use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

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

impl Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rating::Unrated => "unrated".fmt(f),
            Rating::One => "one star".fmt(f),
            Rating::Two => "two stars".fmt(f),
            Rating::Three => "three stars".fmt(f),
            Rating::Four => "four stars".fmt(f),
            Rating::Five => "five stars".fmt(f),
        }
    }
}

pub struct Cullfile {
    /// Map an image file name to its rating
    ratings: HashMap<PathBuf, Rating>,

    base_path: PathBuf,
}

impl Cullfile {
    /// Load the contents of the current directory's cullfile.
    pub fn load(path: impl AsRef<Path>) -> Self {
        let mut ratings = HashMap::new();
        let base_path: PathBuf = path.as_ref().to_path_buf();

        // Get the current working directory
        let Ok(file) = std::fs::read_to_string(path.as_ref().to_path_buf().join(".cullfile"))
        else {
            return Self { ratings, base_path };
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

            ratings.insert(PathBuf::from_str(&file_path).unwrap(), rating);
        }

        Self { ratings, base_path }
    }

    // Serialize self and save to ".cullfile" in the current working directory, overwriting it if it already exists
    pub fn save(&self) {
        let mut file = std::fs::File::create(self.base_path.join(".cullfile")).unwrap();

        for (path, rating) in self.ratings.iter() {
            if !matches!(rating, Rating::Unrated) {
                writeln!(file, "{} {}", *rating as isize, path.to_string_lossy()).unwrap();
            }
        }
    }

    pub fn set_rating(&mut self, image_file_name: impl AsRef<Path>, rating: Rating) {
        self.ratings
            .insert(image_file_name.as_ref().to_path_buf(), rating);
    }

    pub fn get_rating(&self, image_file_name: impl AsRef<Path>) -> Rating {
        self.ratings
            .get(image_file_name.as_ref())
            .map(|r| *r)
            .unwrap_or(Rating::Unrated)
    }
}
