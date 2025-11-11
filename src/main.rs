use std::iter::zip;
use std::path::Path;
use std::time::Instant;
use std::char;
use std::ops::RangeInclusive;
use std::fs;

use clap::Parser;
use image::{ImageBuffer, imageops::invert};
use log::*;
use terminal_size::Width;
use colored::Colorize;

mod core;
use core::*;

use crate::core::blocks::{create_blocks_color, create_blocks_luma};

const CHAR_ASPECT: f32 = 2.0;

#[derive(Clone, Copy, Debug)]
struct CharInfo {
    char: char,
    brightness: u8,
}

fn main() {
    let args = Args::parse();

    if args.debug {
        match simple_logger::init_with_level(log::Level::Debug) {
            Ok(_) => debug!("Logger initialized."),
            Err(e) => println!("WARNING: Failed to initialize logger: {}", e),
        }
    } else {
        match simple_logger::init_with_level(log::Level::Info) {
            Ok(_) => debug!("Logger initialized."),
            Err(e) => println!("WARNING: Failed to initialize logger: {}", e),
    }
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

    // create rgb copy of image
    let mut img_color: ImageBuffer<image::Rgb<u8>, Vec<u8>> = ImageBuffer::new(1, 1);
    if args.color {
        let start_convert_rgb = Instant::now();
        img_color = img.clone().into_rgb8();
        debug!(
            "Converting image to rgb took {}ms.",
            start_convert_rgb.elapsed().as_millis()
        );
    }

    // convert image to grayscale
    let start_convert_luka = Instant::now();
    let mut img = img.into_luma8();
    debug!(
        "Converting image to grayscale took {}ms.",
        start_convert_luka.elapsed().as_millis()
    );

    // inversion of image
    if args.invert {
        let image_invert_start = Instant::now();
        invert(&mut img);
        debug!(
            "Inverted image in {}µs.",
            image_invert_start.elapsed().as_micros()
        );
    }

    debug!("Image dimensions: {:?}", img.dimensions());
    let (img_width, img_height) = img.dimensions();
    debug!("Char Range: {}-{}", *char_range.start(), *char_range.end());

    let terminal_width = match terminal_size::terminal_size() {
        Some((Width(w), _)) => w,
        None => {
            error!("Error getting terminal size.");
            std::process::exit(1);
        }
    };

    debug!("The terminal is {} chars wide", terminal_width);

    let (block_widths, block_heights, chars_width) = blocks::calculate_block_sizes(
        img_width as u16,
        img_height as u16,
        args.scale,
        terminal_width,
        CHAR_ASPECT,
    );

    // i spent a lot of time on a version that was only 100ms faster than the current one but scrapped it.
    let create_blocks_start = Instant::now();
    let blocks = create_blocks_luma(&block_widths, &block_heights, &img);
    debug!(
        "Created {} blocks in {}ms.",
        blocks.len(),
        create_blocks_start.elapsed().as_millis()
    );
    let mut blocks_color: Vec<Vec<u8>> = Vec::new();
    if args.color {
        let create_block_color_start = Instant::now();
        blocks_color = create_blocks_color(&block_widths, &block_heights, &img_color);
        debug!(
            "Created {} color blocks in {}ms.",
            blocks_color.len(),
            create_block_color_start.elapsed().as_millis()
        )
    }

    let font_bytes: Vec<u8>;
    let font_slice: &[u8] = if !args.font.is_empty() {
        match fs::read(&args.font) {
            Ok(f) => {
                font_bytes = f;
                &font_bytes
            },
            Err(e) => {
                error!("Failed to load font from {}: {}", args.font, e);
                std::process::exit(1)
            }
        }
    } else {
        include_bytes!("../font/unifont.otf") as &[u8]
    };

    let font = fontdue::Font::from_bytes(font_slice, fontdue::FontSettings::default()).unwrap();

    let mut char_infos: Vec<CharInfo> = Vec::new();
    let char_render_start = Instant::now();
    for character in char_range {
        let charac = match std::char::from_u32(character) {
            Some(c) => {
                if !c.is_control() {
                    c
                } else {
                    continue;
                }
            }
            None => {
                continue;
            }
        };

        let (_metrics, bitmap) = font.rasterize(charac, 17.0);
        let bitmap = bitmap.iter().map(|f| *f as u64);
        if bitmap.len() < 1 {
            continue;
        }
        char_infos.push(CharInfo {
            char: charac,
            brightness: (bitmap.clone().sum::<u64>() / bitmap.len() as u64) as u8,
        });
    }

    debug!(
        "Rendered {} characters in {}µs.",
        char_infos.len(),
        char_render_start.elapsed().as_micros()
    );

    let start_process_blocks = Instant::now();
    let mut final_str: String = String::new();
    for block in &blocks {
        let avg = if block.is_empty() {
            0
        } else {
            (block.iter().map(|b| *b as u64).sum::<u64>() / block.len() as u64) as u8
        };
        if let Some(closest_char) = find_similar(avg, &char_infos) {
            final_str.push(closest_char);
        } else {
            final_str.push(' ');
        }
    }

    debug!(
        "Processed {} blocks in {}ms.",
        blocks.len(),
        start_process_blocks.elapsed().as_millis()
    );

    let mut color_str = String::new();
    if args.color {
        let start_process_blocks_color = Instant::now();
        for (idx, (letter, block)) in zip(final_str.chars(), blocks_color).enumerate() {
            if block.len() >= 3 {
                let chunks: Vec<_> = block.chunks(3).collect();
                let mut averages = Vec::new();
                for i in 0..chunks[0].len() {
                    let sum: f32 = chunks.iter().map(|chunk| chunk[i] as f32).sum();
                    averages.push(sum / chunks.len() as f32);
                }
                color_str += &letter.to_string().truecolor(averages[0] as u8, averages[1] as u8, averages[2] as u8).to_string();
            } else {
                color_str += &letter.to_string();
            }
            if idx > 0 && idx % chars_width as usize == 0 {
                color_str.push('\n');
            }
        }
        final_str = color_str;
        debug!(
            "Processed {} colored blocks in {}ms.",
            blocks.len(),
            start_process_blocks_color.elapsed().as_millis()
        );
    } else {
        let mut formatted = String::new();
        for (i, c) in final_str.chars().enumerate() {
            if i > 0 && i % chars_width as usize == 0 {
                formatted.push('\n');
            }
            formatted.push(c);
        }
        final_str = formatted;
    }

    println!("{}", final_str);
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// image path
    #[arg(required = true)]
    image_path: String,
    /// unicode char range as "start-end"
    #[arg(short, long, default_value = "32-126")]
    char_range: String,
    /// scale at which to display the image at in the command line
    #[arg(short, long, default_value_t = 1.0)]
    scale: f64,
    /// prints debug messages
    #[arg(long, default_value_t = false)]
    debug: bool,
    /// prints the image in color using ansi escape codes
    #[arg(long, default_value_t = false)]
    color: bool,
    /// lets the user provide a custom opentype or truetype font for processing
    #[arg(long, hide_default_value = true, default_value = "")]
    font: String,
    /// inverts image for processing (color will not be inverted when using --color)
    #[arg(long, default_value_t = false)]
    invert: bool,
    /// stretches the contrast of the image to potentially improve results
    #[arg(long, default_value_t = false)]
    stretch_contrast: bool,
}

fn parse_char_range(char_range: String) -> Result<RangeInclusive<u32>, String> {
    if char_range.matches("-").count() != 1 {
        return Err(
            "Invalid char_range format. Please use \"start-end\" (e.g., \"32-126\").".to_string(),
        );
    }

    let split_range: Vec<&str> = char_range.split("-").map(|s| s.trim()).collect();

    // probably very inefficient in terms of code length
    let range_start: u32 = match split_range.first() {
        Some(s) => match s.parse::<u32>() {
            Ok(start) => start,
            Err(_) => return Err("Invalid start of range.".to_string()),
        },
        None => {
            return Err("No start value in range.".to_string());
        }
    };

    let range_end: u32 = match split_range.get(1) {
        Some(s) => match s.parse::<u32>() {
            Ok(end) => end,
            Err(_) => return Err("Invalid end of range.".to_string()),
        },
        None => {
            return Err("No end value in range.".to_string());
        }
    };

    Ok(range_start..=range_end)
}

fn find_similar(target: u8, arr: &[CharInfo]) -> Option<char> {
    arr.iter()
        .min_by_key(|x| x.brightness.abs_diff(target))
        .map(|x| x.char)
}
