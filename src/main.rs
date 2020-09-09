use std::env;
use std::fs;
use std::io;

use pakhi::lexer;
use pakhi::parser;
use pakhi::interpreter;

fn main() {
    match src_path() {
        Ok(filename) => {
            println!("Source file: {}", filename);
            match src_string(&filename) {
                Ok(src_string) => {
                    println!("{}", src_string);
                    let src_chars: Vec<char> = src_string.chars().collect();
                    let tokens = lexer::tokenize(src_chars);
                    //for t in &tokens {
                    //    println!("{:#?}", t);
                    //}
                    let ast_tree = parser::parse(tokens);
                    // println!("Ast : {:#?}", ast_tree);

                    println!();
                    println!("Interpreter");
                    println!("____________");
                    interpreter::run(ast_tree);
                },
                Err(e) => eprintln!("{}", e),
            }
        },
        Err(e) => eprintln!("{}", e),
    }
}

fn src_path() -> Result<String, &'static str> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        Err("Needs src filename.")
    } else if args.len() > 2 {
        Err("Only one filename required.")
    } else {
        Ok(args[1].clone())
    }
}

fn src_string(filename: &str) -> Result<String, io::Error> {
    match fs::read_to_string(filename) {
        Ok(src_string) => Ok(src_string),
        Err(e) => Err(e)
    }
}