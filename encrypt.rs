// encrypt.rs - Encrypt a binary file with AES-256-CBC using a password from --key.
// Usage: encrypt --key <password> --output <output_file> --bin <input_binary>
// Silent: no output, exits with 0 on success, 1 on error.

use aes::Aes256;
use cbc::{Encryptor, cipher::{BlockEncryptMut, KeyIvInit}};
use sha2::{Sha256, Digest};
use rand::RngCore;
use std::fs;
use std::env;
use std::process;

type Aes256CbcEnc = Encryptor<Aes256>;

fn derive_key(password: &str) -> [u8; 32] {
    let hash = Sha256::digest(password.as_bytes());
    hash.into()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut key = None;
    let mut output = None;
    let mut bin = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--key" => {
                i += 1;
                if i < args.len() {
                    key = Some(args[i].clone());
                    i += 1;
                } else {
                    process::exit(1);
                }
            }
            "--output" => {
                i += 1;
                if i < args.len() {
                    output = Some(args[i].clone());
                    i += 1;
                } else {
                    process::exit(1);
                }
            }
            "--bin" => {
                i += 1;
                if i < args.len() {
                    bin = Some(args[i].clone());
                    i += 1;
                } else {
                    process::exit(1);
                }
            }
            _ => {
                process::exit(1);
            }
        }
    }
    let key = match key { Some(k) => k, None => process::exit(1) };
    let output = match output { Some(o) => o, None => process::exit(1) };
    let bin = match bin { Some(b) => b, None => process::exit(1) };

    let plaintext = match fs::read(&bin) {
        Ok(d) => d,
        Err(_) => process::exit(1),
    };
    let key_bytes = derive_key(&key);
    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);
    let cipher = Aes256CbcEnc::new(&key_bytes.into(), &iv.into());
    let mut ciphertext = plaintext;
    let block_count = (ciphertext.len() + 15) / 16;
    let padded_len = block_count * 16;
    let padding_len = padded_len - ciphertext.len();
    ciphertext.extend(std::iter::repeat(padding_len as u8).take(padding_len));
    cipher.encrypt_blocks_mut(
        unsafe {
            std::slice::from_raw_parts_mut(
                ciphertext.as_mut_ptr() as *mut [u8; 16],
                block_count,
            )
        }
    );
    let mut final_data = iv.to_vec();
    final_data.append(&mut ciphertext);
    if fs::write(&output, &final_data).is_err() {
        process::exit(1);
    }
}
