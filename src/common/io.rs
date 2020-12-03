pub trait IO {
    fn new() -> Self;
    fn print(&mut self, m: &str);
    fn println(&mut self, m: &str);
    fn error(&mut self, m: &str);
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
}

pub struct MockIO {
    print: Vec<String>,
    println: Vec<String>,
    error: Vec<String>,
    expected_print: Vec<String>,
    expected_println: Vec<String>,
    expected_error: Vec<String>,
}

impl MockIO {
    fn expect_print(&mut self, m: &str) {
        self.expected_print.push(String::from(m));
    }

    fn expect_println(&mut self, m: &str) {
        self.expected_println.push(String::from(m));
    }

    fn expect_error(&mut self, m: &str) {
        self.expected_error.push(String::from(m));
    }

    fn assert_all_true(&self) -> bool {
        self.expected_print.eq(&self.print) && self.expected_println.eq(&self.println) &&
            self.expected_error.eq(&self.expected_error)
    }
}

impl IO for MockIO {
    fn new() -> RealIO {
        RealIO
    }

    fn print(&mut self, m: &str) {
        self.print.push(String::from(m));
    }

    fn println(&mut self, m: &str) {
        self.println.push(String::from(m));
    }

    fn error(&mut self, m: &str) {
        self.error.push(String::from(m));
    }
}