pub mod frontend;
pub mod backend;
pub mod common;

use crate::frontend::{lexer, parser};
use crate::backend::interpreter;
use crate::common::io::IO;
use crate::common::pakhi_error::PakhiErr;

pub fn start_pakhi<T: IO>(main_module_path: String, io: &mut T) -> Result<(), PakhiErr>{
    //println!("Source file: {}", filename);
    match io.read_src_code_from_file(&main_module_path) {
        Ok(src_string) => {
            // println!("{}", src_string);
            let src_chars: Vec<char> = src_string.chars().collect();
            let tokens = lexer::tokenize(src_chars, main_module_path.clone());
            //println!("{:#?}", tokens);
            let ast_tree = parser::parse(main_module_path, tokens)?;
            //println!("Ast : {:#?}", ast_tree);

            // println!();
            // println!("Interpreter");
            // println!("____________");
            interpreter::run(ast_tree);
        },
        Err(e) => eprintln!("{}", e),
    }
    Ok(())
}