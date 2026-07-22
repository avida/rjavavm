#![allow(unused)]

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
mod class_loader;
mod java_class;
mod attributes;
mod utils;
mod errors;

use clap::{Parser, CommandFactory};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	/// Load and print class file information
	#[arg(short = 'p', long = "print", value_name = "FILE")]
	print: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cli = Cli::parse();

	if let Some(path) = cli.print {
		let path_str = path.to_str().ok_or("Invalid path")?;
		match class_loader::class_loader::load(path_str) {
			Ok(jc) => println!("{}", jc),
			Err(e) => {
				eprintln!("Error loading class: {}", e);
				std::process::exit(1);
			}
		}
	} else {
		let mut cmd = Cli::command();
		cmd.print_help()?;
		println!();
	}

	Ok(())
}
