#[derive(Debug)]
pub enum PakhiErr {
    // Every tuple is (line_number, file_path, err_message)
    SyntaxError(usize, String, String),
    TypeError(usize, String, String),
    RuntimeError(usize, String, String),
}