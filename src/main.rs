use std::io::{self};

mod decode_byte;
mod disassemble;
mod hex;
use hex::Session;

fn main() -> io::Result<()> {
    let session = Session::new("../example.o", 16, 2).unwrap();
    //   session.dump();
    // session.list_occurrences_string(".rela.debug_info");

    Ok(())
}
