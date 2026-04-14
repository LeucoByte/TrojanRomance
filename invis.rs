// invis.rs - Encode/decode any binary to/from invisible Unicode characters (U+200B, U+200C)
// Compile: rustc invis.rs -o invis
// Usage: invis --encode <input> --output <output>
//        invis --decode <input> --output <output>
// Silent: no output on success or error.

use std::fs;
use std::path::PathBuf;
use std::process;

const BIT0: char = '\u{200B}';
const BIT1: char = '\u{200C}';

fn encode_bytes(data: &[u8]) -> String {
    let mut bits = Vec::with_capacity(data.len() * 8);
    for &byte in data {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1);
        }
    }
    bits.into_iter()
        .map(|b| if b == 0 { BIT0 } else { BIT1 })
        .collect()
}

fn decode_string(invisible_str: &str) -> Result<Vec<u8>, ()> {
    let trimmed = invisible_str.trim();
    let without_brackets = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    let mut bits = Vec::new();
    for ch in without_brackets.chars() {
        match ch {
            BIT0 => bits.push(0),
            BIT1 => bits.push(1),
            c if c.is_whitespace() => continue,
            _ => return Err(()),
        }
    }
    if bits.len() % 8 != 0 {
        return Err(());
    }
    let mut bytes = Vec::with_capacity(bits.len() / 8);
    for chunk in bits.chunks(8) {
        let mut byte = 0u8;
        for &bit in chunk {
            byte = (byte << 1) | bit;
        }
        bytes.push(byte);
    }
    Ok(bytes)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        process::exit(1);
    }
    let mut mode = None;
    let mut input = None;
    let mut output = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--encode" => {
                mode = Some("encode");
                i += 1;
                if i < args.len() {
                    input = Some(PathBuf::from(&args[i]));
                    i += 1;
                }
            }
            "--decode" => {
                mode = Some("decode");
                i += 1;
                if i < args.len() {
                    input = Some(PathBuf::from(&args[i]));
                    i += 1;
                }
            }
            "--output" => {
                i += 1;
                if i < args.len() {
                    output = Some(PathBuf::from(&args[i]));
                    i += 1;
                }
            }
            _ => process::exit(1),
        }
    }
    let mode = match mode {
        Some(m) => m,
        None => process::exit(1),
    };
    let input = match input {
        Some(p) => p,
        None => process::exit(1),
    };
    let output = match output {
        Some(p) => p,
        None => process::exit(1),
    };

    match mode {
        "encode" => {
            let data = match fs::read(&input) {
                Ok(d) => d,
                Err(_) => process::exit(1),
            };
            let encoded = encode_bytes(&data);
            if fs::write(&output, encoded.as_bytes()).is_err() {
                process::exit(1);
            }
        }
        "decode" => {
            let invisible = match fs::read_to_string(&input) {
                Ok(s) => s,
                Err(_) => process::exit(1),
            };
            let decoded = match decode_string(&invisible) {
                Ok(d) => d,
                Err(_) => process::exit(1),
            };
            if fs::write(&output, &decoded).is_err() {
                process::exit(1);
            }
        }
        _ => process::exit(1),
    }
}
