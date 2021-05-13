use super::bit_utils;
use colored::*;
use libflate::zlib::Decoder;
use std::{fmt, io::Read};

#[derive(Debug)]
pub struct Png {
    pub pixels: Vec<Vec<Color>>,
}

impl fmt::Display for Png {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        for row in &self.pixels {
            for y in row {
                str = format!(
                    "{}{}",
                    str,
                    "\u{2588}\u{2588}".truecolor(y.red, y.green, y.blue)
                );
            }
            str = format!("{}{}", str, "\n");
        }
        write!(f, "{}", str)
    }
}

#[derive(Debug)]
enum ColorType {
    GreyScale,
    TrueColor,
    IndexedColour,
    GreyScaleWithAlpha,
    TrueColorWithAlpha,
}

#[derive(Debug)]
struct ImageHeader {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: ColorType,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    fn is_white(&self) -> bool {
        self.red == 255 && self.green == 255 && self.blue == 255
    }
}

#[derive(Debug)]
struct ChunkHeader {
    chunk_length: u32,
    kind: ChunkType,
}

#[derive(Debug, PartialEq)]
pub enum ChunkType {
    IHDR,
    // ICCP,
    PLTE,
    IDAT,
    IEND,
    NotRelevant,
}

struct Parser {
    reader: bit_utils::ByteReader,
    image_header: Option<ImageHeader>,
    palette: Vec<Color>,
}

impl Parser {
    fn new(file: &str) -> Self {
        Self {
            reader: bit_utils::ByteReader::new(file),
            image_header: None,
            palette: Vec::new(),
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            reader: bit_utils::ByteReader {
                content: bytes.to_vec(),
                cursor_pos: 0,
            },
            image_header: None,
            palette: Vec::new(),
        }
    }

    fn read_next_chunk_header(&mut self) -> ChunkHeader {
        ChunkHeader {
            chunk_length: self.reader.read_4_byte_value(),
            kind: match self.reader.read_next(4) {
                [73u8, 72, 68, 82] => ChunkType::IHDR,
                // [105u8, 67, 67, 80] => ChunkType::ICCP, // probably not relevant
                [80u8, 76, 84, 69] => ChunkType::PLTE,
                [73u8, 68, 65, 84] => ChunkType::IDAT,
                [73u8, 69, 78, 68] => ChunkType::IEND,
                _ => ChunkType::NotRelevant,
            },
        }
    }

    fn parse_signature(&mut self) {
        let signature = [137u8, 80, 78, 71, 13, 10, 26, 10];
        if self.reader.read_next(8) != &signature {
            panic!("This is not a png.")
        };
    }

    fn parse_ihdr(&mut self) -> ImageHeader {
        let image_header = ImageHeader {
            width: self.reader.read_4_byte_value(),
            height: self.reader.read_4_byte_value(),
            bit_depth: self.reader.read_next_byte(),
            color_type: match self.reader.read_next_byte() {
                0 => ColorType::GreyScale,
                2 => ColorType::TrueColor,
                3 => ColorType::IndexedColour,
                4 => ColorType::GreyScaleWithAlpha,
                6 => ColorType::TrueColorWithAlpha,
                _ => panic!("ColorType is not known."),
            },
            compression_method: self.reader.read_next_byte(),
            filter_method: self.reader.read_next_byte(),
            interlace_method: self.reader.read_next_byte(),
        };
        self.skip_crc();
        image_header
    }

    fn parse_plte(&mut self, chunk_header: &ChunkHeader) -> Vec<Color> {
        let mut palette: Vec<Color> = Vec::new();
        for _i in 0..chunk_header.chunk_length / 3 {
            let color = Color {
                red: self.reader.read_next_byte(),
                green: self.reader.read_next_byte(),
                blue: self.reader.read_next_byte(),
            };
            palette.push(color);
        }
        self.skip_crc();
        palette
    }

    fn parse_idat(&mut self, chunk_header: &ChunkHeader) -> Vec<Vec<Color>> {
        let mut decoded_data = decompress(self.reader.read_next(chunk_header.chunk_length));
        let mut pixels: Vec<Vec<Color>> = Vec::new();
        let width = self.image_header.as_ref().unwrap().width;
        let height = self.image_header.as_ref().unwrap().height;
        self.skip_crc();
        let byte_width_to_take = byte_num_from_pixel_width(width);
        for _ in 0..height {
            let filter_byte: u8 = decoded_data.remove(0);
            if filter_byte != 0 {
                panic!("Not handled filter! {}", filter_byte)
            }
            let mut scanline: Vec<u8> = decoded_data.drain(0..byte_width_to_take).collect();
            scanline = scanline
                .into_iter()
                .flat_map(|val| bit_utils::byte_to_4bits(val))
                .collect::<Vec<u8>>()
                .drain(0..width as usize)
                .collect();

            pixels.push(
                scanline
                    .into_iter()
                    .map(|idx| self.palette[idx as usize])
                    .collect(),
            );
        }
        pixels
    }

    fn skip_chunk(&mut self, chunk_header: &ChunkHeader) {
        self.reader.skip_next(chunk_header.chunk_length + 4); // garbage bytes, CRC 4 bytes
    }

    fn skip_crc(&mut self) {
        self.reader.skip_next(4); // CRC 4 bytes
    }

    fn parse_png(&mut self) -> Png {
        let mut png = Png { pixels: Vec::new() };
        self.parse_signature();
        let mut chunk_header = self.read_next_chunk_header();
        while chunk_header.kind != ChunkType::IEND {
            match chunk_header.kind {
                ChunkType::IHDR => {
                    self.image_header = Some(self.parse_ihdr());
                }
                ChunkType::PLTE => self.palette = self.parse_plte(&chunk_header),
                ChunkType::IDAT => png.pixels = self.parse_idat(&chunk_header),
                ChunkType::NotRelevant => self.skip_chunk(&chunk_header),
                _ => self.skip_chunk(&chunk_header),
            }
            chunk_header = self.read_next_chunk_header();
        }
        png
    }
}

fn decompress(compressed_data: &[u8]) -> Vec<u8> {
    let mut decoded_data: Vec<u8> = Vec::new();
    let mut decoder = Decoder::new(&compressed_data[..]).unwrap();
    decoder.read_to_end(&mut decoded_data).unwrap();
    decoded_data
}

fn byte_num_from_pixel_width(width: u32) -> usize {
    if width % 2 == 0 {
        (width / 2) as usize
    } else {
        ((width + 1) / 2) as usize
    }
}

impl Png {
    #![allow(dead_code)]
    pub fn new(file: &str) -> Self {
        let mut parser = Parser::new(file);
        parser.parse_png()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut parser = Parser::from_bytes(bytes);
        parser.parse_png()
    }

    fn col(&self, n: usize) -> Vec<Color> {
        let mut col_pixels: Vec<Color> = Vec::new();
        for row in &self.pixels {
            col_pixels.push(row[n]);
        }
        col_pixels
    }

    fn row(&self, n: usize) -> &[Color] {
        &self.pixels[n][..]
    }

    pub fn width(&self) -> usize {
        self.pixels[0].len()
    }

    pub fn height(&self) -> usize {
        self.pixels.len()
    }

    pub fn remove_white_lines(&mut self) {
        let mut r = 0;
        while r < self.height() {
            if self.row(r).iter().all(|pixel| pixel.is_white()) {
                self.pixels.remove(r);
            } else {
                r += 1;
            }
        }

        let mut c = 0;
        while c < self.width() {
            if self.col(c).iter().all(|pixel| pixel.is_white()) {
                for row in self.pixels.iter_mut() {
                    row.remove(c);
                }
            } else {
                c += 1;
            }
        }
    }
}
