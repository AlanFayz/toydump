use std::fmt::Write;
use std::sync::{Arc, Mutex};

use crate::decode_byte::*;

const ARM64_INSTRUCTION_SIZE: usize = 4;
const ARM64_INSTRUCTION_ENDIAN: Endianness = Endianness::LittleEndian;

fn does_bit_pattern_match(pattern: &str, number: u32) -> bool {
    let mask = pattern
        .chars()
        .rev()
        .enumerate()
        .fold(0_u32, |curr, (i, c)| match c {
            '1' => curr | (1 << i),
            '0' => curr | (1 << i),
            'x' => curr,
            _ => panic!("invalid pattern {pattern}"),
        });

    let target = pattern
        .chars()
        .rev()
        .enumerate()
        .fold(0_u32, |curr, (i, c)| match c {
            '1' => curr | (1 << i),
            '0' => curr,
            'x' => curr,
            _ => panic!("invalid pattern {pattern}"),
        });

    (number & mask) == target
}

fn format_register(is_w: bool, r: u32) -> String {
    if r == 31 {
        return "sp".to_string();
    }

    if is_w {
        return format!("w{}", r);
    }

    return format!("x{}", r);
}

fn format_instruction_rd_imm(name: &str, is_w: bool, rd: u32, imm: u32) -> String {
    format!("{} {:>2}, #{}", name, format_register(is_w, rd), imm)
}

fn format_instruction_rd_rn_imm(name: &str, is_w: bool, rd: u32, rn: u32, imm: u32) -> String {
    format!(
        "{} {:>2}, {}, #{}",
        name,
        format_register(is_w, rd),
        format_register(is_w, rn),
        imm
    )
}

fn format_instruction_rd_rn_imm_offset(name: &str, s: bool, rd: u32, rn: u32, imm: u32) -> String {
    format!(
        "{} {:>2}, [{}, #{}]",
        name,
        format_register(s, rd),
        format_register(s, rn),
        imm
    )
}

fn decode_aarch64_data_processing_one_source(instruction: u32) -> String {
    let sf = instruction >> 31;
    let opc = (instruction >> 21) & 0x03;
    let rd = instruction & 0x1F;
    let imm16 = (instruction >> 5) & 0xFFFF;

    if sf == 0 || (rd & 0b11) == rd {
        return format_instruction_rd_imm("unallocated", false, rd, imm16);
    }

    if rd == 0x1F {
        if opc == 0 {
            return format_instruction_rd_imm("autiasppc", false, rd, imm16);
        } else {
            return format_instruction_rd_imm("autibsppc", false, rd, imm16);
        }
    }

    return format_instruction_rd_imm("unknown", false, rd, imm16);
}

fn decode_aarch64_data_processing_immediate_pc_rel_addressing(instruction: u32) -> String {
    let op = instruction >> 31;
    let immlo = (instruction >> 29) & 0x3;
    let immhi = (instruction >> 5) & 0x7FFFF;

    let data = (immhi << 2) | immlo;
    let rd = instruction & 0xF;

    if op == 0 {
        return format_instruction_rd_imm("adr", false, rd, data);
    }

    return format_instruction_rd_imm("adrp", false, rd, data);
}

fn decode_aarch64_data_processing_immediate_add_sub(instruction: u32) -> String {
    let sf = (instruction >> 31) & 1;
    let op = (instruction >> 30) & 1;
    let s = (instruction >> 29) & 1;
    let rd = instruction & 0x1F;
    let rn = (instruction >> 5) & 0x1F;
    let imm12 = (instruction >> 10) & 0xFFF;

    let inst_name = match op {
        0 => "add",
        _ => "sub",
    };

    let update_flags = if s == 1 { "s" } else { "" };

    format_instruction_rd_rn_imm(
        format!("{inst_name}{update_flags}").as_str(),
        sf == 0,
        rd,
        rn,
        imm12,
    )
}

fn decode_aarch64_data_processing_immediate(instruction: u32) -> String {
    let op0 = (instruction >> 29) & 0x3;
    let op1 = (instruction >> 22) & 0xF;

    if op0 == 0x3 && does_bit_pattern_match("111x", op1) {
        return decode_aarch64_data_processing_one_source(instruction);
    }

    if does_bit_pattern_match("00xx", op1) {
        return decode_aarch64_data_processing_immediate_pc_rel_addressing(instruction);
    }

    if does_bit_pattern_match("010x", op1) {
        return decode_aarch64_data_processing_immediate_add_sub(instruction);
    }

    String::from("data_processing_immediate")
}

fn decode_aarch64_data_processing_register_extended_add_sub(instruction: u32) -> String {
    {
        let _sf = instruction >> 31;
        let _op = (instruction >> 30) & 1;
        let _s = (instruction >> 29) & 1;
        let _opt = (instruction >> 22) & 0x3;
        let _rm = (instruction >> 16) & 0x1F;
    }

    String::from("data_processing_register_extended_add_sub")
}

fn decode_aarch64_data_processing_register_three_source(instruction: u32) -> String {
    String::from("data_processing_register_three_source")
}

fn decode_aarch64_data_processing_register(instruction: u32) -> String {
    let _op0 = (instruction >> 30) & 1;
    let op1 = (instruction >> 28) & 1;
    let op2 = (instruction >> 21) & 0xF;
    let _op3 = (instruction >> 10) & 0x3F;

    if op1 == 0 {
        if does_bit_pattern_match("1xx1", op2) {
            return decode_aarch64_data_processing_register_extended_add_sub(instruction);
        }
    }

    if op1 == 1 {
        if does_bit_pattern_match("1xxx", op2) {
            return decode_aarch64_data_processing_register_three_source(instruction);
        }
    }

    String::from("data_processing_register")
}

fn decode_aarch64_sme(_instruction: u32) -> String {
    String::from("sme")
}

fn decode_aarch64_sve(_instruction: u32) -> String {
    String::from("sve")
}

fn decode_aarch64_load_store_load_register_literal(instruction: u32) -> String {
    let opc = instruction >> 30;
    let vr = (instruction >> 26) & 1;
    let imm19 = (instruction >> 5) & 0x7FFFF;
    let rd = instruction & 0x1F;

    String::from("load_store_load_register_literal")
}

fn decode_aarch64_load_store_register_unsigned_immediate(instruction: u32) -> String {
    let size = instruction >> 30;
    let v = (instruction >> 26) & 1;
    let opc = (instruction >> 22) & 0x3;
    let imm12 = (instruction >> 10) & 0xFFF;
    let rn = (instruction >> 5) & 0x1F;
    let rd = instruction & 0x1F;

    if v != 0 {
        return String::from("vectorized_load_store_register_unsigned_immediate");
    }

    let inst_name = match opc {
        0 => "str",
        1 => "ldr",
        _ => "unknown",
    };

    let scale = match size {
        0 => 1,
        1 => 2,
        2 => 4,
        3 => 8,
        _ => 1,
    };

    let use_w_register = size != 3;
    format_instruction_rd_rn_imm_offset(inst_name, use_w_register, rd, rn, imm12 * scale)
}

fn decode_aarch64_load_store(instruction: u32) -> String {
    let op0 = instruction >> 28;
    let _op1 = (instruction >> 26) & 1;
    let op2 = (instruction >> 10) & 0x7FFF;

    if does_bit_pattern_match("xx01", op0)
        && does_bit_pattern_match(format!("0{}", "x".repeat(14)).as_str(), op2)
    {
        return decode_aarch64_load_store_load_register_literal(instruction);
    }

    if does_bit_pattern_match("xx11", op0)
        && does_bit_pattern_match(format!("1{}", "x".repeat(14)).as_str(), op2)
    {
        return decode_aarch64_load_store_register_unsigned_immediate(instruction);
    }

    String::from("load_store")
}

fn decode_aarch64_instruction(instruction: &[u8]) -> String {
    let instruction: [u8; ARM64_INSTRUCTION_SIZE] = instruction.try_into().unwrap();
    let instruction = get_value::<u32>(ARM64_INSTRUCTION_ENDIAN, &instruction);

    //NOTE: see the arm architecture reference manual for  better explanations
    let op0 = instruction >> 31;
    let op1 = (instruction >> 25) & 0xF;

    if op1 == 0 && op0 == 0 {
        return String::from("reserved");
    }

    if op1 == 0 && op0 == 1 {
        return decode_aarch64_sme(instruction);
    }

    if op1 == 0b0010 {
        return decode_aarch64_sve(instruction);
    }

    if does_bit_pattern_match("00x1", op1) {
        return String::from("unallocated");
    }

    if does_bit_pattern_match("100x", op1) {
        return decode_aarch64_data_processing_immediate(instruction);
    }

    if does_bit_pattern_match("x101", op1) {
        return decode_aarch64_data_processing_register(instruction);
    }

    if does_bit_pattern_match("x1x0", op1) {
        return decode_aarch64_load_store(instruction);
    }

    String::from("instruction")
}

pub fn print_aarch64_disassembly(bytes: &[u8], output_string: Arc<Mutex<String>>) {
    let str = bytes
        .chunks_exact(ARM64_INSTRUCTION_SIZE)
        .map(|instruction| {
            return (
                decode_aarch64_instruction(instruction),
                get_value::<u32>(ARM64_INSTRUCTION_ENDIAN, instruction),
            );
        })
        .enumerate()
        .map(|(i, s)| format!("{:<5}          0x{:08X}          {}\n", i, s.1, s.0))
        .collect::<String>();

    writeln!(output_string.lock().unwrap(), "{str}");
}
