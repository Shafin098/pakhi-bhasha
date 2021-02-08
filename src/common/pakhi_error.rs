#[derive(Debug)]
pub enum PakhiErr {
    // Every tuple is (line_number, file_path, err_message)
    SyntaxError(u32, String, String),
    TypeError(u32, String, String),
    RuntimeError(u32, String, String),
    UnexpectedError(String), // Here only string will contain error message
}