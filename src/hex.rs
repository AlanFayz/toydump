use colored::*;
use std::fs::File;
use std::io::Read;

use crate::disassemble::*;

pub struct Session {
    data: Vec<u8>,
    column_count: usize,
    group_count: usize,
}

impl Session {
    pub fn new(filename: &str, column_count: usize, group_count: usize) -> Option<Session> {
        let file = File::open(filename);

        if file.is_err() {
            return Option::None;
        }

        let mut data = Vec::<u8>::new();
        let result = file.unwrap().read_to_end(&mut data);

        if result.is_err() {
            return Option::None;
        }

        let header = ElfHeader::new(&data)?;

        Some(Session {
            data,
            column_count,
            group_count,
        })
    }

    fn format_byte(&self, mut index: usize, byte: &u8) -> String {
        index += 1;

        let format_str = if index % self.group_count == 0 {
            format!("{:02X} ", byte)
        } else {
            format!("{:02X}", byte)
        };

        if byte.is_ascii_graphic() {
            return format_str.green().to_string();
        }

        format_str
    }

    fn format_hex_line(&self, bytes: &[u8], row_index: usize) -> String {
        let offset = row_index * self.column_count;
        let byte_stream = bytes
            .iter()
            .enumerate()
            .map(|(i, byte)| self.format_byte(i, byte))
            .collect::<String>();

        let str_stream = bytes
            .iter()
            .map(|byte| {
                if byte.is_ascii_graphic() {
                    return (*byte as char).to_string().green().to_string();
                }

                return ".".to_owned();
            })
            .collect::<String>();

        format!("{:08X} {} {}\n", offset, byte_stream, str_stream)
    }

    fn get_sep(&self) -> String {
        let count =
            size_of::<u64>() + self.column_count * self.group_count + self.column_count * 2 + 1;

        let mut sep = String::new();

        for _ in 0..count {
            sep += "-";
        }

        sep.magenta().to_string()
    }

    pub fn dump(&self) {
        let contents = self
            .data
            .chunks(self.column_count)
            .enumerate()
            .map(|(i, chunk)| self.format_hex_line(chunk, i))
            .collect::<String>();

        println!("{}", contents);
    }

    pub fn list_occurrences(&self, bytes: &[u8]) {
        let sep = self.get_sep();

        let occurrences = self
            .data
            .windows(bytes.len())
            .enumerate()
            .filter(|(_, window)| bytes == *window)
            .map(|(i, window)| {
                let raw_index = i;
                let row_index = raw_index / self.column_count;

                let count = window.len() / 16 + 1;
                let mut window_string = String::new();

                for c in 0..count {
                    let start = (row_index + c) * self.column_count;
                    window_string += self
                        .format_hex_line(
                            &self.data[start..(start + self.column_count).min(self.data.len())],
                            row_index + c,
                        )
                        .as_str();
                }

                window_string
            })
            .map(|s| format!("{}\n{}", sep, s))
            .collect::<String>();

        println!("{}", occurrences);
    }

    pub fn list_occurrences_string(&self, s: &str) {
        let bytes = s.to_owned().into_bytes();
        self.list_occurrences(&bytes);
    }
}
