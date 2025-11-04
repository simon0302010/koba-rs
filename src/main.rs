use std::ops::RangeInclusive;
use std::path::Path;
use std::time::Instant;

use clap::Parser;
use image::{self, GenericImageView};
use log::*;
use terminal_size::Width;

mod core;
use core::*;

use crate::core::blocks::create_blocks;

const CHAR_ASPECT: f32 = 2.0;
const SCALE: f64 = 1.0;

fn main() {
    let args = Args::parse();

    match simple_logger::init() {
        Ok(_) => debug!("Logger initialized."),
        Err(e) => println!("WARNING: Failed to initialize logger: {}", e),
    }

    let char_range = match parse_char_range(args.char_range) {
        Ok(char_range) => char_range,
        Err(e) => {
            error!("Failed to parse char range: {}", e);
            std::process::exit(1);
        }
    };

    let img_path = Path::new(&args.image_path);
    if !img_path.exists() {
        error!("File does not exist.");
        std::process::exit(1);
    }

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
    let (img_width, img_height) = img.dimensions();
    info!("Char Range: {}-{}", *char_range.start(), *char_range.end());

    let terminal_width = match terminal_size::terminal_size() {
        Some((Width(w), _)) => w,
        None => {
            error!("Error getting terminal size.");
            std::process::exit(1);
        }
    };

    info!("The terminal is {} chars wide", terminal_width);

    let (block_widths, block_heights, chars_width) = blocks::calculate_block_sizes(
        img_width as u16,
        img_height as u16,
        SCALE,
        terminal_width,
        CHAR_ASPECT,
    );

    let create_blocks_start = Instant::now();
    let blocks = create_blocks(&block_widths, &block_heights, &img);
    info!(
        "Created {} blocks in {}ms.",
        blocks.len(),
        create_blocks_start.elapsed().as_millis()
    );
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// image path
    #[arg(required = true)]
    image_path: String,
    /// Unicode character range to use for ASCII art, specified as "start-end" (e.g., "32-126").
    /// Both start and end should be integer Unicode code points.
    /// This determines which characters are used to represent image brightness.
    #[arg(short, long)]
    char_range: String,
}

fn parse_char_range(char_range: String) -> Result<RangeInclusive<i32>, String> {
    if char_range.matches("-").count() != 1 {
        return Err(
            "Invalid char_range format. Please use \"start-end\" (e.g., \"32-126\").".to_string(),
        );
    }

    let split_range: Vec<&str> = char_range.split("-").map(|s| s.trim()).collect();

    // probably very inefficient in terms of code length
    let range_start: i32 = match split_range.first() {
        Some(s) => match s.parse::<i32>() {
            Ok(start) => start,
            Err(_) => return Err("Invalid start of range.".to_string()),
        },
        None => {
            return Err("No start value in range.".to_string());
        }
    };

    let range_end: i32 = match split_range.get(1) {
        Some(s) => match s.parse::<i32>() {
            Ok(end) => end,
            Err(_) => return Err("Invalid end of range.".to_string()),
        },
        None => {
            return Err("No end value in range.".to_string());
        }
    };

    Ok(range_start..=range_end)
}
