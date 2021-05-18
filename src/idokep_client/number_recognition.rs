use super::png::{Color, Png};
use std::error;

pub struct NumberRecognition;

impl NumberRecognition {
    pub fn extract_number(mut png: Png) -> Result<f64, Box<dyn error::Error>> {
        png.remove_white_lines();
        Ok(get_number(png))
    }
}

fn get_number(image: Png) -> f64 {
    let mut digits = String::new();
    let (char, mut pos) = get_char(0, &image);
    digits.push(char);
    while pos < image.width() {
        let (char, new_pos) = get_char(pos, &image);
        pos = new_pos;
        digits.push(char);
    }
    digits.parse().unwrap()
}

fn get_char(starting_pos: usize, image: &Png) -> (char, usize) {
    let chars = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', '-'];
    let mut results = std::collections::HashMap::new();
    for &c in chars.iter() {
        let char_png: Png = Png::from_bytes(char_image(c));
        if char_png.width() > image.width() - starting_pos {
            continue;
        }
        let mut diff = 0;
        for (row, row_vals) in char_png.pixels.iter().enumerate() {
            for (col, pixel) in row_vals.iter().enumerate() {
                diff += pixel_diff(pixel, &image.pixels[row][col + starting_pos])
            }
        }
        results.insert(c, diff);
    }
    let &result_char = results.iter().min_by(|a, b| a.1.cmp(b.1)).unwrap().0;
    (
        result_char,
        starting_pos + Png::from_bytes(char_image(result_char)).width(),
    )
}

fn char_image(c: char) -> &'static [u8] {
    match c {
        '0' => include_bytes!("char_images/0.png"),
        '1' => include_bytes!("char_images/1.png"),
        '2' => include_bytes!("char_images/2.png"),
        '3' => include_bytes!("char_images/3.png"),
        '4' => include_bytes!("char_images/4.png"),
        '5' => include_bytes!("char_images/5.png"),
        '6' => include_bytes!("char_images/6.png"),
        '7' => include_bytes!("char_images/7.png"),
        '8' => include_bytes!("char_images/8.png"),
        '9' => include_bytes!("char_images/9.png"),
        '-' => include_bytes!("char_images/minus.png"),
        '.' => include_bytes!("char_images/point.png"),
        _ => panic!("Unhandled character"),
    }
}

fn pixel_diff(pixel1: &Color, pixel2: &Color) -> u32 {
    let red_diff =
        (pixel1.red as i32 - pixel2.red as i32) * (pixel1.red as i32 - pixel2.red as i32);
    let green_diff =
        (pixel1.green as i32 - pixel2.green as i32) * (pixel1.green as i32 - pixel2.green as i32);
    let blue_diff =
        (pixel1.blue as i32 - pixel2.blue as i32) * (pixel1.blue as i32 - pixel2.blue as i32);
    (red_diff + green_diff + blue_diff) as u32
}
