mod aarch64_disassembler;
use std::fmt::Write;
use std::sync::{Arc, Mutex};

use aarch64_disassembler::*;

use crate::decode_byte::*;

const ELF_MAGIC_NUMBER: u8 = 0x7F;
const ELF_IDENTITY: &str = "ELF";

enum BitFormat {
    Bit32,
    Bit64,
}

enum OSAbi {
    Linux,
    SystemV,
    Unsupported,
}

enum InstructionSet {
    Arm64,
    Unsupported,
}

#[allow(dead_code)]
pub struct ElfHeader {
    data: Vec<u8>,
    format: BitFormat,
    endianness: Endianness,
    abi: OSAbi,
    instruction_set: InstructionSet,
    section_header_offset: u64,
    section_header_entry_size: u16,
    section_header_entry_count: u16,
    section_header_names_index: u16,
    output_string: Arc<Mutex<String>>,
}

impl ElfHeader {
    pub fn new(bytes: &[u8], output_string: Arc<Mutex<String>>) -> Option<ElfHeader> {
        if bytes.len() < 64 {
            let _ = writeln!(output_string.lock().unwrap(), "invalid header");
            return Option::None;
        }

        let header_info = &bytes[0..64];

        if header_info[0] != ELF_MAGIC_NUMBER {
            let _ = writeln!(output_string.lock().unwrap(), "invalid header");
            return Option::None;
        }

        if header_info[1..4] != ELF_IDENTITY.to_owned().into_bytes() {
            let _ = writeln!(output_string.lock().unwrap(), "invalid header");
            return Option::None;
        }

        let format = if header_info[4] == 1 {
            BitFormat::Bit32
        } else {
            BitFormat::Bit64
        };

        if matches!(format, BitFormat::Bit32) {
            let _ = writeln!(
                output_string.lock().unwrap(),
                "currently unsupported on 32 bit systems"
            );
            return Option::None;
        }

        let endianness = if header_info[5] == 1 {
            Endianness::LittleEndian
        } else {
            Endianness::BigEndian
        };

        let abi = match header_info[7] {
            0x03 => OSAbi::Linux,
            0x00 => OSAbi::SystemV,
            _ => OSAbi::Unsupported,
        };

        if matches!(abi, OSAbi::Unsupported) {
            let _ = writeln!(
                output_string.lock().unwrap(),
                "unsupported abi {:02X}",
                header_info[7]
            );

            return Option::None;
        }

        let value = get_value::<u16>(endianness, &bytes[0x12..0x12 + 2]);
        let instruction_set = match value {
            0xB7 => InstructionSet::Arm64,
            _ => InstructionSet::Unsupported,
        };

        if matches!(instruction_set, InstructionSet::Unsupported) {
            let _ = writeln!(
                output_string.lock().unwrap(),
                "unsupported instruction set {:04X}",
                value
            );
            return Option::None;
        }

        let section_header_offset = get_value::<u64>(endianness, &bytes[0x28..0x28 + 8]);
        let section_header_entry_size = get_value::<u16>(endianness, &bytes[0x3A..0x3A + 2]);
        let section_header_entry_count = get_value::<u16>(endianness, &bytes[0x3C..0x3C + 2]);
        let section_header_names_index = get_value::<u16>(endianness, &bytes[0x3E..0x3E + 2]);

        let header = ElfHeader {
            data: bytes.to_vec(),
            format,
            endianness,
            abi,
            instruction_set,
            section_header_offset,
            section_header_entry_size,
            section_header_entry_count,
            section_header_names_index,
            output_string,
        };

        Some(header)
    }

    fn string_from_shstrtab(&self, offset: usize) -> String {
        let shstrtab_section_header_offset = self.section_header_offset
            + self.section_header_names_index as u64 * self.section_header_entry_size as u64;

        let shstrtab_section_header_offset = shstrtab_section_header_offset as usize;

        let shstrtab_offset = get_value::<u64>(
            self.endianness,
            &self.data
                [shstrtab_section_header_offset + 0x18..shstrtab_section_header_offset + 0x18 + 8],
        ) as usize;

        let mut name = String::new();
        let mut i: usize = 0;

        loop {
            let byte = self.data[shstrtab_offset + offset + i];
            i += 1;

            if byte == 0 {
                break;
            }

            name.push(byte as char);
        }

        name
    }

    fn dump_section_code(&self, header_index: usize) {
        let header_offset = self.section_header_offset
            + header_index as u64 * self.section_header_entry_size as u64;

        let header_offset = header_offset as usize;

        let flags = get_value::<u64>(
            self.endianness,
            &self.data[header_offset + 0x08..header_offset + 0x08 + 8],
        );

        let name_offset = get_value::<u32>(
            self.endianness,
            &self.data[header_offset..header_offset + 4],
        ) as usize;

        let name = self.string_from_shstrtab(name_offset);
        if name != ".text" {
            return;
        }

        let _ = writeln!(self.output_string.lock().unwrap(), "section {name}");

        let section_offset = get_value::<u64>(
            self.endianness,
            &self.data[header_offset + 0x18..header_offset + 0x18 + 8],
        ) as usize;

        let section_size = get_value::<u64>(
            self.endianness,
            &self.data[header_offset + 0x20..header_offset + 0x20 + 8],
        ) as usize;

        if matches!(self.instruction_set, InstructionSet::Arm64) {
            print_aarch64_disassembly(
                &self.data[section_offset..section_offset + section_size],
                self.output_string.clone(),
            );
        }
    }

    pub fn dump_disassembly(&self) {
        for i in 0..self.section_header_entry_count {
            self.dump_section_code(i as usize);
        }
    }
}
