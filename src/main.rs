use std::{error::Error, path::PathBuf};

use clap::Parser;
use elf::{endian::AnyEndian, ElfBytes};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // We get more arguments from `cargo avr-build` so we ignore them here.
    padding: Option<String>,
    /// File to check
    #[arg(short = 'f', long)]
    elf_file: PathBuf,

    /// If true this program will return an error code and print to stderr, if
    /// it did not fit into the assigned space. Other errors like parsing errors
    /// or io errors can not be ignored with this flag.
    #[arg(short = 'f', long)]
    error: bool,

    /// The max amount of program memory on the arduino.
    #[arg(short = 'p', long, default_value = "32256")]
    max_program_memory: u64,
    /// The max amount of dynamic memory on the arduino.
    #[arg(short = 'd', long, default_value = "2048")]
    max_dynamic_memory: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let path = &args.elf_file;
    let file_data = std::fs::read(path)?;
    let file_data = file_data.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(file_data)?;
    let mut program_memory = 0;
    let mut dynamic_memory = 0;
    for a in file.segments().unwrap() {
        if a.p_flags == 5 {
            program_memory += a.p_filesz;
        } else if a.p_flags == 6 {
            dynamic_memory += a.p_filesz;
        }
    }
    let program_usage_percent =
        (program_memory as f64 / args.max_program_memory as f64 * 100.0).ceil() as u64;
    let dynamic_usage_percent =
        (dynamic_memory as f64 / args.max_dynamic_memory as f64 * 100.0).ceil() as u64;
    println!("This sketch uses {program_memory} bytes ({program_usage_percent}%) of program memory. The maximum is {} bytes.", args.max_program_memory);
    println!("This sketch uses {dynamic_memory} bytes ({dynamic_usage_percent}%) of dynamic memory. The maximum is {} bytes.", args.max_dynamic_memory);
    if args.error {
        if program_usage_percent > 100 || dynamic_usage_percent > 100 {
            return Err("Uses too much memory!".into());
        }
    }
    Ok(())
}
