use std::io::Error;

pub trait IO {
    fn new() -> Self;
    fn print(&mut self, m: &str);
    fn println(&mut self, m: &str);
    fn error(&mut self, m: &str);
    fn read_src_code_from_file(&mut self, filename: &str) -> Result<String, std::io::Error>;
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

    fn error(&mut self, m: &str) {
        eprintln!("{}", m);
    }

    fn read_src_code_from_file(&mut self, file_path: &str) -> Result<String, Error> {
        match std::fs::read_to_string(file_path) {
            Ok(src_string) => Ok(src_string),
            Err(e) => Err(e)
        }
    }
}

#[derive(Debug, Clone)]
pub struct MockIO {
    print: Vec<String>,
    println: Vec<String>,
    error: Vec<String>,
    op_order: Vec<String>,
    expected_print: Vec<String>,
    expected_println: Vec<String>,
    expected_error: Vec<String>,
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

    pub fn expect_error(&mut self, m: &str) {
        self.expected_error.push(String::from(m));
        self.expected_op_order.push(String::from("error"));
    }

    pub fn assert_all_true(&self) -> bool {
        //println!("{:#?}", self);
        self.expected_print.eq(&self.print) &&
        self.expected_println.eq(&self.println) &&
        self.expected_error.eq(&self.expected_error) &&
        self.expected_op_order.eq(&self.op_order)
    }
}

impl IO for MockIO {
    fn new() -> MockIO {
        MockIO {
            print: Vec::new(),
            println: Vec::new(),
            error: Vec::new(),
            op_order: Vec::new(),
            expected_print: Vec::new(),
            expected_println: Vec::new(),
            expected_error: Vec::new(),
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

    fn error(&mut self, m: &str) {
        self.error.push(String::from(m));
        self.op_order.push(String::from("error"));
    }

    fn read_src_code_from_file(&mut self, _filename: &str) -> Result<String, Error> {
        unimplemented!()
    }
}