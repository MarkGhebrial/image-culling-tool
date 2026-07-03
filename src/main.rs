mod app;
mod cullfile;
mod image;

use ::image::EncodableLayout;
use ::image::RgbImage;
use app::*;
use eframe::egui::ImageData;
use eframe::egui::TextureId;
use eframe::egui::TextureOptions;
use eframe::egui::load::SizedTexture;
use eframe::epaint::TextureManager;
use eframe::{egui, epaint};

use std::ops::Deref;
use std::sync::Arc;
use std::{env, path::Path, process::exit};

use crate::cullfile::Cullfile;
use crate::image::load_images;

fn main() {
    let args: Vec<String> = env::args().collect();

    let executable_path = env::current_exe().unwrap();
    let executable_name = executable_path.file_name().unwrap().to_string_lossy();

    let path_argument = match args.get(1) {
        Some(p) => p,
        None => {
            print_usage(&executable_name);
            exit(-1);
        }
    };

    let path = match Path::new(path_argument).canonicalize() {
        Ok(p) => p,
        Err(e) => {
            println!("Path \"{path_argument}\" is not valid: {e}");
            exit(-2);
        }
    };

    println!("{}", path.to_str().unwrap());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_app_id("cull tool"), //.with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "cull tool",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let texture_manager = cc.egui_ctx.tex_manager();

            let images = match load_images(&path, false, texture_manager.clone()) {
                Ok(images) => images,
                Err(e) => {
                    println!("Error loading images: {:?}", e);
                    exit(-1);
                }
            };

            for image in &images {
                println!(
                    "Found image at {}",
                    image.path_relative_to_cullfile.to_str().unwrap()
                );
            }

            Ok(Box::new(MyApp::new(Cullfile::load(&path), images)))
        }),
    )
    .unwrap();
}

struct TexBox<T> {
    image: Arc<T>,
    texture_manager: Arc<egui::mutex::RwLock<TextureManager>>,
    texture_id: TextureId,
}
impl<T> TexBox<T>
where
    T: ImageData,
{
    pub fn new(image: Arc<T>, texture_manager: Arc<egui::mutex::RwLock<TextureManager>>) -> Self {
        let texture_id = texture_manager.write().alloc(
            "texbox texture".to_owned(),
            image.clone(),
            TextureOptions::default(),
        );

        Self {
            image,
            texture_manager,
            texture_id,
        }
    }

    pub fn get_image(&self) -> Arc<T> {
        self.image.clone()
    }

    pub fn get_texture_id(&self) -> TextureId {
        self.texture_id.clone()
    }

    pub fn get_sized_texture(&self) -> SizedTexture {
        SizedTexture {
            id: self.texture_id.clone(),
            size: [self.image.width() as f32, self.image.height() as f32].into(),
        }
    }
}
impl<T> Drop for TexBox<T> {
    fn drop(&mut self) {
        // If there's only one reference to the image remaining
        if Arc::strong_count(&self.image) == 1 {
            self.texture_manager.write().free(self.texture_id.clone());
        }
    }
}

#[derive(Debug)]
struct ImageWrapper(pub RgbImage);

impl epaint::ImageData for ImageWrapper {
    fn size(&self) -> [usize; 2] {
        [
            self.0.dimensions().0 as usize,
            self.0.dimensions().1 as usize,
        ]
    }

    fn width(&self) -> usize {
        self.size()[0]
    }

    fn height(&self) -> usize {
        self.size()[1]
    }

    fn pixel_type(&self) -> epaint::image::PixelType {
        epaint::image::PixelType::Rgb
    }

    fn data(&self) -> &[u8] {
        &self.0.as_bytes()
    }
}

impl Deref for ImageWrapper {
    type Target = RgbImage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn print_usage(name: &str) {
    println!("Usage: {name} <path>");
}
