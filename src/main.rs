use std::env;
use std::path::Path;

use image::{self, GenericImageView};

fn main() {
    let executable = env::args().next().expect("No executable path found");
    let args: Vec<String> = env::args().skip(1).collect();

    let img_path = match args.first() {
        Some(path) => {
            let file_path = Path::new(path);
            if !file_path.exists() {
                println!("File does not exist.");
                std::process::exit(1);
            }
            file_path
        }
        None => {
            println!("Usage: {} IMAGE_PATH", executable);
            std::process::exit(1);
        }
    };

    let img = match image::open(img_path) {
        Ok(img) => img,
        Err(e) => {
            println!("Couldn't load image: {}", e);
            std::process::exit(1);
        }
    };

    println!("Image dimensions: {:?}", img.dimensions());
}
