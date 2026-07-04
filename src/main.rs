mod app;
mod cullfile;
mod image;
mod image_wrapper;
mod zoom_image_widget;

use app::*;
use eframe::egui;

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

    let images = match load_images(&path, false) {
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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_app_id("cull tool"), //.with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "cull tool",
        options,
        Box::new(|cc| {
            Ok(Box::new(MyApp::new(
                Cullfile::load(&path),
                images,
                &cc.egui_ctx,
            )))
        }),
    )
    .unwrap();
}

fn print_usage(name: &str) {
    println!("Usage: {name} <path>");
}
