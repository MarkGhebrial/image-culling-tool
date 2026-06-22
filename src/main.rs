mod app;
mod image;

use app::*;

use std::{env, fs, path::Path, process::exit};

use ::image::ImageReader;
use ::image::ImageDecoder;

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
        },
    };   

    // println!("Starting {executable_name} at {path:?}");

    scan_directory(&path);

}

/// Given a path, return all the images in that path
fn scan_directory(path: &Path) -> () {
    if path.is_file() {
        let mut image_decoder = ImageReader::open(path).expect("reeee").into_decoder().expect("yeeet");

        let exif = image_decoder.exif_metadata().unwrap();

        if let Some(bytes) = exif {
            for byte in bytes {
                print!("{}", byte as char);
            }
        }

        // println!("EXIF data:");
        // println!("{:#?}", exif);
    }

    ()
}

fn print_usage(name: &str) {
    println!(
        "Usage: {name} <path>"
    );
}