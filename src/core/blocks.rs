use std::cmp;

pub fn calculate_block_sizes(
    width: u16,
    height: u16,
    scale: f64,
    terminal_width: u16,
    char_aspect: f32,
) -> (Vec<u16>, Vec<u16>, u16) {
    let min_block_width = 10.0 / char_aspect;
    let min_block_height = 10.0;

    let scale = scale.min(1.0);

    let max_chars_width_by_block_width = (width as f32 / min_block_width) as u16;
    let max_chars_width_by_block_height = (height as f32 / min_block_height) as u16;

    let max_chars_width = cmp::min(
        max_chars_width_by_block_width,
        max_chars_width_by_block_height,
    );

    let chars_width: u16 = ((terminal_width as f64 * scale) as u16)
        .min(terminal_width as u16)
        .min(max_chars_width as u16)
        .max(1);

    let chars_height: u16 = (((height as u32 * chars_width as u32) as f64 / width as f64)
        / char_aspect as f64)
        .ceil() as u16;

    let mut block_widths = vec![width / chars_width; chars_width as usize];
    for i in 0..(width % chars_width as u16) {
        block_widths[i as usize] += 1;
    }

    let mut block_heights = vec![height / chars_height; chars_height as usize];
    for i in 0..(height % chars_height) {
        block_heights[i as usize] += 1;
    }

    (block_widths, block_heights, chars_width)
}
