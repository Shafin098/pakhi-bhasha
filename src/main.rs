use std::env;
use pakhi::start_pakhi;
use pakhi::common::io::{RealIO, IO};

fn main() {
    let main_module_path = get_main_module_path();
    match main_module_path {
        Ok(path) => {
            let mut io = RealIO::new();
            if let Err(err) = start_pakhi(path, &mut io) {
                io.panic(err);
            }
        },
        Err(e) => eprintln!("Err: {}", e),
    }
}

fn get_main_module_path() -> Result<String, &'static str> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        Err("Needs src filename.")
    } else if args.len() > 2 {
        Err("Only one filename required.")
    } else {
        if args[1].ends_with(".pakhi") {
            Ok(args[1].clone())
        } else {
            Err("Source file must have .pakhi extension.")
        }
    }
}