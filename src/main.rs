use std::env;

use pakhi::lexer;
use pakhi::parser;
use pakhi::interpreter;
use pakhi::common::io;
use pakhi::common::io::IO;

fn main() {
    match main_module_path_provided() {
        Ok(main_module_path) => {
            //println!("Source file: {}", filename);
            let mut io = io::RealIO::new();
            match io.read_src_code_from_file(&main_module_path) {
                Ok(src_string) => {
                    // println!("{}", src_string);
                    let src_chars: Vec<char> = src_string.chars().collect();
                    let tokens = lexer::tokenize(src_chars);
                    //println!("{:#?}", tokens);
                    //for t in &tokens {
                    //    println!("{:#?}", t);
                    //}
                    let ast_tree = parser::parse(main_module_path, tokens);
                    //println!("Ast : {:#?}", ast_tree);

                    // println!();
                    // println!("Interpreter");
                    // println!("____________");
                    interpreter::run(ast_tree);
                },
                Err(e) => eprintln!("{}", e),
            }
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn main_module_path_provided() -> Result<String, &'static str> {
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