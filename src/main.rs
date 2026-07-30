#![allow(unused)]

mod loader;
mod utils;
mod vm;

use crate::loader::utils::utils::lookup_class_file;
use crate::vm::runtime::Runtime;
use crate::{loader::class_loader::class_loader::load, vm::errors::errors::RunTimeError};
use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Load and print class file information
    #[arg(short = 'p', long = "print", value_name = "FILE")]
    print: Option<std::path::PathBuf>,

    /// Run a class by its fully-qualified name (e.g. com.example.Main)
    #[arg(short = 'r', long = "run", value_name = "CLASS")]
    run: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(path) = cli.print {
        if let Some(file_path) = lookup_class_file(path.to_str().unwrap()) {
            match load(file_path.to_str().unwrap()) {
                Ok(jc) => println!("{}", jc),
                Err(e) => {
                    eprintln!("Error loading class: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Cannot find class  {}", path.to_str().unwrap());
        }
    } else if let Some(class_name) = cli.run {
        // Load/init runtime with a fully-qualified class name; Runtime will
        // resolve the .class file via `lookup_class_file` internally.
        match Runtime::load_and_init() {
            Some(mut runtime) => match runtime.run(&class_name) {
                Ok(()) => println!("Execution finished successfully"),
                Err(e) => {
                    eprintln!("Runtime error: {}", e);
                    std::process::exit(1);
                }
            },
            None => {
                eprintln!("Class file not found for {}", class_name);
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
