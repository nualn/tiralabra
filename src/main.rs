use std::fs::File;

use clap::{Parser, Subcommand};
use rsa::keys::{Key, KeyPair};

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate,
    Encrypt {
        #[arg(short, long)]
        in_path: String,
        #[arg(short, long)]
        out_path: String,
        #[arg(short, long)]
        key_path: String,
    },
    Decrypt {
        #[arg(short, long)]
        in_path: String,
        #[arg(short, long)]
        out_path: String,
        #[arg(short, long)]
        key_path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate => {
            let keys = KeyPair::generate();
            match keys.write_to_file() {
                Ok(_) => (),
                Err(e) => panic!("Failed to write keys to file: {:?}", e),
            };
        }
        Commands::Encrypt {
            in_path,
            out_path,
            key_path,
        } => {
            let key = match Key::from_file(key_path) {
                Ok(key) => key,
                Err(e) => panic!("Failed to read key from file: {}", e),
            };

            let mut in_file = match File::open(in_path) {
                Ok(file) => file,
                Err(e) => panic!("Failed to open file: {:?}", e),
            };
            let mut out_file = match File::create(out_path) {
                Ok(file) => file,
                Err(e) => panic!("Failed to open file: {:?}", e),
            };

            match key.encrypt(&mut in_file, &mut out_file) {
                Ok(_) => (),
                Err(e) => panic!("Failed to encrypt file: {:?}", e),
            };
        }
        Commands::Decrypt {
            in_path,
            out_path,
            key_path,
        } => {
            let mut key = match Key::from_file(key_path) {
                Ok(key) => key,
                Err(e) => panic!("Failed to read key from file: {}", e),
            };

            let mut in_file = match File::open(in_path) {
                Ok(file) => file,
                Err(e) => panic!("Failed to open file: {:?}", e),
            };
            let mut out_file = match File::create(out_path) {
                Ok(file) => file,
                Err(e) => panic!("Failed to open file: {:?}", e),
            };

            match key.decrypt(&mut in_file, &mut out_file) {
                Ok(_) => (),
                Err(e) => panic!("Failed to decrypt file: {:?}", e),
            };
        }
    };
}
