use crate::common::pakhi_error::PakhiErr;

pub trait IO {
    fn new() -> Self;
    fn print(&mut self, m: &str);
    fn println(&mut self, m: &str);
    fn read_src_code_from_file(&mut self, file_path: &str) -> Result<String, std::io::Error> {
        match std::fs::read_to_string(file_path) {
            Ok(src_string) => Ok(src_string),
            Err(e) => Err(e)
        }
    }
    fn panic(&mut self, err: PakhiErr);
}

pub struct RealIO;

impl IO for RealIO {
    fn new() -> RealIO {
        RealIO
    }

    fn print(&mut self, m: &str) {
        print!("{}", m);
    }

    fn println(&mut self, m: &str) {
        println!("{}", m);
    }

    fn panic(&mut self, err: PakhiErr) {
        match err {
            PakhiErr::SyntaxError(line, file_name, err_message) => {
                eprintln!("SyntaxError: {}", err_message);
                eprintln!("    at file: {}, line: {}", file_name, line);
                std::process::exit(1);
            },
            PakhiErr::RuntimeError(line, file_name, err_message) => {
                eprintln!("RuntimeError: {}", err_message);
                eprintln!("    at file: {}, line: {}", file_name, line);
                std::process::exit(1);
            },
            PakhiErr::TypeError(line, file_name, err_message) => {
                eprintln!("TypeError: {}", err_message);
                eprintln!("    at file: {}, line: {}", file_name, line);
                std::process::exit(1);
            },
            PakhiErr::UnexpectedError(err_message) => {
                eprintln!("UnexpectedError: {}", err_message);
                std::process::exit(1);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockIO {
    print: Vec<String>,
    println: Vec<String>,
    panic: Vec<PakhiErr>,
    op_order: Vec<String>,
    expected_print: Vec<String>,
    expected_println: Vec<String>,
    expected_panic: Vec<PakhiErr>,
    expected_op_order: Vec<String>,
}

impl MockIO {
    pub fn expect_print(&mut self, m: &str) {
        self.expected_print.push(String::from(m));
        self.expected_op_order.push(String::from("print"));
    }

    pub fn expect_println(&mut self, m: &str) {
        self.expected_println.push(String::from(m));
        self.expected_op_order.push(String::from("println"));
    }

    pub fn expect_panic(&mut self, err: PakhiErr) {
        self.expected_panic.push(err);
        self.expected_op_order.push(String::from("panic"));
    }

    pub fn assert_all_true(&self) {
        for (i, _)in self.print.iter().enumerate() {
            assert_eq!(self.expected_print[i], self.print[i])
        }
        for (i, _)in self.println.iter().enumerate() {
            assert_eq!(self.expected_println[i], self.println[i])
        }
        for (i, _)in self.panic.iter().enumerate() {
            assert_eq!(self.expected_panic[i], self.panic[i])
        }
        for (i, _)in self.op_order.iter().enumerate() {
            assert_eq!(self.expected_op_order[i], self.op_order[i])
        }
    }
}

impl IO for MockIO {
    fn new() -> MockIO {
        MockIO {
            print: Vec::new(),
            println: Vec::new(),
            panic: Vec::new(),
            op_order: Vec::new(),
            expected_print: Vec::new(),
            expected_println: Vec::new(),
            expected_panic: Vec::new(),
            expected_op_order: Vec::new(),
        }
    }

    fn print(&mut self, m: &str) {
        self.print.push(String::from(m));
        self.op_order.push(String::from("print"));
    }

    fn println(&mut self, m: &str) {
        self.println.push(String::from(m));
        self.op_order.push(String::from("println"));
    }

    fn panic(&mut self, err: PakhiErr) {
        self.panic.push(err);
        self.op_order.push("panic".to_string());
    }
}