use std::{convert::TryInto, fs};

#[derive(Debug)]
pub struct ByteReader {
    pub content: Vec<u8>,
    pub cursor_pos: u32,
}

impl ByteReader {
    pub fn new(file: &str) -> Self {
        Self {
            content: fs::read(file).expect("File couldn't be read."),
            cursor_pos: 0,
        }
    }

    pub fn read_next(&mut self, n: u32) -> &[u8] {
        let next_cursor_pos = self.cursor_pos + n;
        let chunk = &self.content[self.cursor_pos as usize..next_cursor_pos as usize];
        self.cursor_pos += n;
        chunk
    }

    pub fn skip_next(&mut self, n: u32) {
        self.cursor_pos += n;
    }

    pub fn read_4_byte_value(&mut self) -> u32 {
        u32::from_be_bytes(self.read_next(4).try_into().expect("it didn't fit"))
    }

    pub fn read_next_byte(&mut self) -> u8 {
        self.read_next(1)[0]
    }
}

pub fn byte_to_4bits(byte: u8) -> Vec<u8> {
    let first = (byte >> 4) & 0x0f;
    let second = byte & 0x0f;
    vec![first, second]
}
