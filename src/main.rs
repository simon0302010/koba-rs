use std::env;
use std::path::Path;
use std::time::Instant;

use image::{self, GenericImageView};
use log::*;

fn main() {
    match simple_logger::init() {
        Ok(_) => debug!("Logger initialized."),
        Err(e) => println!("WARNING: Failed to initialize logger: {}", e),
    }

    let executable = env::args().next().expect("No executable path found");
    let args: Vec<String> = env::args().skip(1).collect();

    let img_path = match args.first() {
        Some(path) => {
            let file_path = Path::new(path);
            if !file_path.exists() {
                error!("File does not exist.");
                std::process::exit(1);
            }
            file_path
        }
        None => {
            error!("Usage: {} IMAGE_PATH", executable);
            std::process::exit(1);
        }
    };

    let image_loading_start = Instant::now();
    let img = match image::open(img_path) {
        Ok(img) => img,
        Err(e) => {
            error!("Failed to load image: {}", e);
            std::process::exit(1);
        }
    };
    debug!(
        "Loading image took {}ms",
        image_loading_start.elapsed().as_millis()
    );

    info!("Image dimensions: {:?}", img.dimensions());
}
