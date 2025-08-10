use std::{
    fs::File,
    io::{self, Error, Write},
    process::Command,
    sync::{Arc, Mutex},
};

mod decode_byte;
mod disassemble;
mod hex;

use clap::Parser;
use hex::Session;

#[derive(Parser)]
#[command(
    name = "toydump",
    about = "A tool to inspect binary files, useful for hex dumping and disassembly viewing."
)]
struct Args {
    #[arg(help = "File to open")]
    filepath: String,

    #[arg(long, help = "Dump hex?")]
    hex: bool,

    #[arg(short, long, help = "Dump disassembly?")]
    disassembly: bool,

    #[arg(long, help = "Optionally choose number of columns for hex dump")]
    columns: Option<u32>,

    #[arg(
        long,
        help = "Optionally choose how many bytes are in one group for hex dump"
    )]
    groups: Option<u32>,

    #[arg(short, long, help = "Search for string in hex")]
    search: Option<String>,

    #[arg(short, long, help = "Optional output filepath")]
    output: Option<String>,

    #[arg(
        short,
        long,
        help = "Open in editor, e.g., --editor code or --editor vim"
    )]
    editor: Option<String>,
}

fn main() {
    let args = Args::parse();
    let output = Arc::new(Mutex::new(String::new()));

    let session = Session::new(
        args.filepath.as_str(),
        args.columns.unwrap_or(16) as usize,
        args.groups.unwrap_or(2) as usize,
        output.clone(),
        args.output.is_none(),
    );

    if session.is_none() {
        return;
    }

    let session = session.unwrap();

    if args.hex {
        session.dump();
    }

    if args.disassembly {
        let elf = session.elf_header();
        if elf.is_some() {
            elf.unwrap().dump_disassembly();
        }
    }

    if args.search.is_some() {
        let search_string = args.search.unwrap();
        session.list_occurrences_string(&search_string);
    }

    if args.output.is_some() {
        let output_filepath = args.output.unwrap();
        let mut file = File::create(output_filepath.clone())
            .expect(format!("failed to create file {output_filepath}").as_str());

        let _ = file.write_all(output.lock().unwrap().as_bytes());

        if let Some(editor) = args.editor.as_ref() {
            let status = Command::new(editor)
                .arg(&output_filepath)
                .stdin(std::process::Stdio::inherit()) // Pass terminal input to vim
                .stdout(std::process::Stdio::inherit()) // Pass terminal output from vim
                .stderr(std::process::Stdio::inherit()) // Pass error output from vim
                .status() // Wait for it to finish
                .expect("failed to run editor");

            println!("Editor exited with status: {}", status);
        }

        return;
    }

    if args.editor.is_some() {
        println!("an output file must be specified in order to open it with an editor");
        return;
    }

    println!("{}", output.lock().unwrap());
}
