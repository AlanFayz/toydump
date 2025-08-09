use crate::decode_byte::*;

const ARM64_INSTRUCTION_SIZE: usize = 4;
const ARM64_INSTRUCTION_ENDIAN: Endianness = Endianness::LittleEndian;

fn format_instruction_rd_imm(name: &str, s: bool, rd: u32, imm: u32) -> String {
    if s {
        format!("{} {:>2}, #{}", name, format!("w{}", rd), imm)
    } else {
        format!("{} {:>2}, #{}", name, format!("x{}", rd), imm)
    }
}

fn format_instruction_rd_rn_imm(name: &str, s: bool, rd: u32, rn: u32, imm: u32) -> String {
    if s {
        format!(
            "{} {:>2}, {}, #{}",
            name,
            if rd < 31 {
                format!("w{}", rd)
            } else {
                "sp".to_owned()
            },
            if rn < 31 {
                format!("w{}", rn)
            } else {
                "sp".to_owned()
            },
            imm
        )
    } else {
        format!(
            "{} {:>2}, {}, #{}",
            name,
            if rd < 31 {
                format!("x{}", rd)
            } else {
                "sp".to_owned()
            },
            if rn < 31 {
                format!("x{}", rn)
            } else {
                "sp".to_owned()
            },
            imm
        )
    }
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

    if op == 0 {
        if s == 1 {
            return format_instruction_rd_rn_imm("adds", s == 1, rd, rn, imm12);
        } else {
            return format_instruction_rd_rn_imm("add", s == 1, rd, rn, imm12);
        }
    }

    if s == 1 {
        return format_instruction_rd_rn_imm("subs", s == 1, rd, rn, imm12);
    } else {
        return format_instruction_rd_rn_imm("sub", s == 1, rd, rn, imm12);
    }
}

fn decode_aarch64_data_processing_immediate(instruction: u32) -> String {
    let op0 = (instruction >> 29) & 0x3;
    let op1 = (instruction >> 22) & 0xF;

    if op0 == 0x3 && (op1 & 0xF) == op1 {
        return decode_aarch64_data_processing_one_source(instruction);
    }

    if op1 & 0b0011 == op1 {
        return decode_aarch64_data_processing_immediate_pc_rel_addressing(instruction);
    }

    if op1 & 0b0101 == op1 {
        return decode_aarch64_data_processing_immediate_add_sub(instruction);
    }

    String::from("unimplemented")
}

fn decode_aarch64_sme(instruction: u32) -> String {
    String::from("sme not implemented")
}

fn decode_aarch64_sve(instruction: u32) -> String {
    String::from("sve not implemented")
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

    if op1 & 0b0011 == op1 {
        return String::from("unallocated");
    }

    if op1 & 0b1001 == op1 {
        return decode_aarch64_data_processing_immediate(instruction);
    }

    String::from("unimplemented")
}

pub fn print_aarch64_disassembly(bytes: &[u8]) {
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

    println!("{}", str);
}
