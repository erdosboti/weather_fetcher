mod bit_utils;
mod png;

use base64::decode;
use png::{Color, Png};
use regex::Regex;

fn main() -> Result<(), ureq::Error> {
    // let mut png = Png::new("src/assets/2.3.png");
    // println!("{}", png);
    // png.remove_white_lines();
    // println!("{}", png);
    // println!("{}", get_number(png));
    let body: String = ureq::get("https://www.idokep.hu/automata/globallhotel")
        .call()?
        .into_string()?;
    // println!("{}", body);

    let re = Regex::new(r#"<th class="">Hőmérséklet</th>\s+<td><img alt="Embedded Image" src="data:image/png;base64,(.+)"> °C</td>"#).unwrap();

    let caps = re.captures(&body).unwrap();
    let text1 = caps.get(1).map_or("", |m| m.as_str());
    // println!("{}", text1);
    let decoded = decode(text1).unwrap();
    let mut png = Png::from_bytes(decoded);
    // println!("{}", png);
    png.remove_white_lines();
    println!("{}", png);
    println!("{}", get_number(png));

    Ok(())
}

fn get_number(image: Png) -> f32 {
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
        let char_png: Png = Png::new(&char_file_name(c));
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
        starting_pos + Png::new(&char_file_name(result_char)).width(),
    )
}

fn char_file_name(c: char) -> String {
    let file_name = match c {
        '.' => "point".to_owned(),
        '-' => "minus".to_owned(),
        _ => c.to_string().to_owned(),
    };
    format!("src/assets/{}.png", file_name)
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
