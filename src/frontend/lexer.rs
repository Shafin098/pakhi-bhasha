use std::collections::HashMap;
use crate::common::pakhi_error::PakhiErr::SyntaxError;
use crate::common::pakhi_error::PakhiErr;

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: Vec<char>,
    pub line: u32,
    pub src_file_path: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Num(f64),
    String(String),
    Identifier,
    If,
    Else,
    Loop,
    Var,        // নাম keyword is TokenKind of Var. Not variable identifier
    Function,
    Plus,
    Minus,
    Multiply,
    Division,
    Remainder,
    At,
    Semicolon,
    Map,
    Comment,
    Comma,
    ParenStart,
    ParenEnd,
    CurlyBraceStart,
    CurlyBraceEnd,
    SquareBraceStart,
    SquareBraceEnd,
    Equal,
    LessThan,
    GreaterThan,
    EqualEqual,
    NotEqual,
    LessThanOrEqual,
    GreaterThanOrEqual,
    And,
    Or,
    Not,
    Bool(bool),
    Break,
    Continue,
    Return,
    Print,
    Import,
    PrintNoEOL,
    EOT, // represents end of token, only needed for parsing to indicate
         // all previous tokens were consumed
}

pub fn tokenize(src: Vec<char>, src_file_path: String) -> Result<Vec<Token>, PakhiErr> {
    let mut current_i = 0;
    let mut line = 1;

    let mut tokens: Vec<Token> = Vec::new();

    while current_i < src.len() {
        // c represents total chars consumed by token t
        // l represents total line consumed by token t
        let (t, c, l) = consume(&src, current_i, line, src_file_path.clone())?;
        if let Some(token) = t {
            tokens.push(token);
        }
        current_i += c;
        line += l;
    }
    tokens.push(Token {
        kind: TokenKind::EOT,
        lexeme: "".chars().collect(),
        line: 0,
        src_file_path,
    });

    Ok(tokens)
}

fn consume(src: &Vec<char>, start: usize, line: u32, src_file_path: String) -> Result<(Option<Token>, usize, u32), PakhiErr> {
    let consumed_char: usize;
    let consumed_line: u32;
    let token: Token;

    match src[start] {
        '-'|'০'|'১'|'২'|'৩'|'৪'|'৫'|'৬'|'৭'|'৮'|'৯' => {
            if src[start+1].is_numeric() || src[start].is_numeric() {
                // negative number, unary '-' operator
                let (val, consumed) = consume_num(src, start, line, &src_file_path)?;

                consumed_char = consumed;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::Num(val),
                    lexeme: src[start..(start+consumed_char)].to_vec(),
                    line: line + consumed_line,
                    src_file_path,
                }
            } else {
                // not a negative number, binary '-' operator or map operator '->' in record

                if src[start+1] == '>' {
                    // map operator '->' in record
                    consumed_char = 2;
                    consumed_line = 0;
                    token = Token {
                        kind: TokenKind::Map,
                        lexeme: src[start..(start+2)].to_vec(),
                        line,
                        src_file_path,
                    }
                } else {
                    // binary '-' operator
                    consumed_char = 1;
                    consumed_line = 0;
                    token = Token {
                        kind: TokenKind::Minus,
                        lexeme: src[start..(start+1)].to_vec(),
                        line,
                        src_file_path,
                    }
                }
            }
        },
        '+' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Plus,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        '*' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Multiply,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        '/' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Division,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        '%' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Remainder,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        '&' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::And,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        '|' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Or,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        '!' => {
            if start < src.len() && src[start+1] == '=' {
                consumed_char = 2;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::NotEqual,
                    lexeme: src[start..(start+2)].to_vec(),
                    line,
                    src_file_path,
                }
            } else {
                consumed_char = 1;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::Not,
                    lexeme: src[start..(start+1)].to_vec(),
                    line,
                    src_file_path,
                }
            }
        },
        '@' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::At,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        '#' => {
            let (char_skipped, lines_skipped) = skip_comment_block(src, start, line, &src_file_path)?;
            consumed_char = char_skipped;
            consumed_line = lines_skipped;
            token = Token {
                kind: TokenKind::Comment,
                lexeme: src[start..(start+char_skipped)].to_vec(),
                line,
                src_file_path,
            }
        },
        ';' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Semicolon,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        ',' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Comma,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        '"' => {
            let (val, consumed) = consume_string(src, start);

            consumed_char = consumed;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::String(val),
                // start + 1 for excluding first " and (start+consumed_char)-1 for excluding last "
                lexeme: src[(start+1)..((start+consumed_char)-1)].to_vec(),
                line: line + consumed_line,
                src_file_path,
            }
        },
        '(' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::ParenStart,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        ')' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::ParenEnd,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        '{' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::CurlyBraceStart,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        '}' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::CurlyBraceEnd,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        '[' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::SquareBraceStart,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        ']' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::SquareBraceEnd,
                lexeme: src[start..(start+1)].to_vec(),
                line,
                src_file_path,
            }
        },
        '=' => {
            if start < src.len() && src[start+1] == '=' {
                consumed_char = 2;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::EqualEqual,
                    lexeme: src[start..(start+2)].to_vec(),
                    line,
                    src_file_path,
                }
            } else {
                consumed_char = 1;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::Equal,
                    lexeme: src[start..(start+1)].to_vec(),
                    line,
                    src_file_path,
                }
            }
        },
        '<' => {
            if start < src.len() && src[start+1] == '=' {
                consumed_char = 2;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::LessThanOrEqual,
                    lexeme: src[start..(start+2)].to_vec(),
                    line,
                    src_file_path,
                }
            } else {
                consumed_char = 1;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::LessThan,
                    lexeme: src[start..(start+1)].to_vec(),
                    line,
                    src_file_path,
                }
            }
        },
        '>' => {
            if start < src.len() && src[start+1] == '=' {
                consumed_char = 2;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::GreaterThanOrEqual,
                    lexeme: src[start..(start+2)].to_vec(),
                    line,
                    src_file_path,
                }
            } else {
                consumed_char = 1;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::GreaterThan,
                    lexeme: src[start..(start+1)].to_vec(),
                    line,
                    src_file_path,
                }
            }
        },
        ' ' | '\r' | '\t' => {
            consumed_char = 1;
            consumed_line = 0;
            return Ok((None, consumed_char, consumed_line));
        },
        '\n' => {
            consumed_char = 1;
            consumed_line = 1;
            return Ok((None, consumed_char, consumed_line));
        },
        _ => {
            // if nothing matches must be an identifier
            let (t, consumed) = consume_identifier(src, start, line, src_file_path);

            consumed_char = consumed;
            consumed_line = 0;
            token = t;
        },
    }

    Ok((Some(token), consumed_char, consumed_line))
}

fn consume_num(src: &Vec<char>, start: usize, line: u32, src_file_path: &str) -> Result<(f64, usize), PakhiErr> {
    assert!(src[start].clone().is_numeric() || src[start] == '-');

    let mut consumed = 0;
    let mut val = 0.0;
    let mut fractional_val = 0.0;

    let mut i = start;
    let is_negative = if src[start] == '-' {
        // skipping negative sign
        consumed += 1;
        i += 1;
        true
    } else {
        false
    };
    let mut in_fractional_part = false;

    while i < src.len() && (src[i].clone().is_numeric() || src[i] == '.') {
        if src[i] == '.' {
            if in_fractional_part {
                return Err(SyntaxError(line, src_file_path.to_string(),
                                       "Number is not properly formatted".to_string()));
            }
            in_fractional_part = true;
            consumed += 1;
            i += 1;
            continue;
        }

        if in_fractional_part {
            fractional_val = (fractional_val * 10.0) + bn_digit_to_en_digit(src[i], line, src_file_path)?;
            consumed += 1;
            i += 1;
        } else {
            val = (val * 10.0) + bn_digit_to_en_digit(src[i], line, src_file_path)?;
            consumed += 1;
            i += 1;
        }
    }
    fractional_val = fractional_val / (10_f64.powf(fractional_val.to_string().len() as f64));

    if is_negative {
        Ok(((val + fractional_val) * -1.0, consumed))
    } else {
        Ok(((val + fractional_val), consumed))
    }
}

fn consume_string(src: &Vec<char>, start: usize) -> (String, usize) {
    assert_eq!('"', src[start]);

    let mut consumed = 0;
    let mut val = String::new();

    let mut i = start + 1;
    while i < src.len() && (src[i].clone() != '"') {
        val.push(src[i]);
        consumed += 1;
        i += 1;
    }
    // adding extra 2 for first " and last "
    consumed += 2;

    (val, consumed)
}

fn consume_identifier(src: &Vec<char>, start: usize, line: u32, src_file_path: String) -> (Token, usize) {
    let mut consumed = 0;
    let mut char_vec: Vec<char>= Vec::new();

    let mut i = start;
    while i < src.len() && is_valid_identifier_char(src[i]) {
        char_vec.push(src[i]);
        consumed += 1;
        i +=1;
    }

    let token = match keyword(&char_vec, line, src_file_path.clone()) {
        Some(t) => t,
        None => Token {
            kind: TokenKind::Identifier,
            lexeme: src[start..(start+consumed)].to_vec(),
            line,
            src_file_path,
        }
    };

    (token, consumed)
}

fn is_valid_identifier_char(c: char) -> bool {
    if c == '-' || c == '_' || c == '/' {
        return true;
    }
    !c.is_ascii_whitespace() && !c.is_ascii_punctuation() && !c.is_ascii_control()
}

fn keyword(char_vec: &Vec<char>, line: u32, src_file_path: String) -> Option<Token> {
    let mut keyword_map: HashMap<Vec<char>, TokenKind> = HashMap::new();
    keyword_map.insert("নাম".chars().collect(), TokenKind::Var);
    keyword_map.insert("যদি".chars().collect(), TokenKind::If);
    keyword_map.insert("অথবা".chars().collect(), TokenKind::Else);
    keyword_map.insert("লুপ".chars().collect(), TokenKind::Loop);
    keyword_map.insert("ফাং".chars().collect(), TokenKind::Function);
    keyword_map.insert("ফেরত".chars().collect(), TokenKind::Return);
    keyword_map.insert("থামাও".chars().collect(), TokenKind::Break);
    keyword_map.insert("আবার".chars().collect(), TokenKind::Continue);
    keyword_map.insert("দেখাও".chars().collect(), TokenKind::Print);
    keyword_map.insert("_দেখাও".chars().collect(), TokenKind::PrintNoEOL);
    keyword_map.insert("সত্য".chars().collect(), TokenKind::Bool(true));
    keyword_map.insert("মিথ্যা".chars().collect(), TokenKind::Bool(false));
    keyword_map.insert("মডিউল".chars().collect(), TokenKind::Import);

    match keyword_map.remove(char_vec) {
        Some(token_kind) => Some(Token {
            kind: token_kind,
            lexeme: char_vec.to_vec(),
            line,
            src_file_path,
        }),
        None => None,
    }
}

fn bn_digit_to_en_digit(digit: char, line: u32, src_file_path: &str) -> Result<f64, PakhiErr> {
    match digit {
        '০' => return Ok(0.0),
        '১' => return Ok(1.0),
        '২' => return Ok(2.0),
        '৩' => return Ok(3.0),
        '৪' => return Ok(4.0),
        '৫' => return Ok(5.0),
        '৬' => return Ok(6.0),
        '৭' => return Ok(7.0),
        '৮' => return Ok(8.0),
        '৯' => return Ok(9.0),
        _ => {
            return Err(SyntaxError(line, src_file_path.to_string(), format!("Cannot convert '{}' to bangla digit", digit)));
        },
    }
}

fn skip_comment_block(src: &Vec<char>, start: usize, line: u32, src_file_path: &str) -> Result<(usize, u32), PakhiErr> {
    let mut char_skipped: usize = 1;
    let mut lines_skipped: u32 = 0;
    while src[start + char_skipped] != '#' {
        if (start + char_skipped + 1) > src.len() - 1 {
            return Err(SyntaxError(line, src_file_path.to_string(), "Comment block wasn't closed".to_string()))
        }
        if src[start + char_skipped] == '\\' && src[start + char_skipped + 1] == '#' {
            // if # escaped with \ skipping this #
            char_skipped += 2;
            continue;
        }
        char_skipped += 1;
        if src[start + char_skipped] == '\n' {
            lines_skipped += 1;
        }
    }
    // skipping last #
    char_skipped += 1;

    Ok((char_skipped, lines_skipped))
}

#[cfg(test)]
mod tests {
    use crate::lexer::{consume_num, keyword, TokenKind, consume_string};

    #[test]
    fn lexer_consume_num_test_1() {
        let digits_1 = vec!['২', '৪', '৫'];

        let (val, consumed) = consume_num(&digits_1, 0, 1, "test.pakhi").unwrap();
        assert_eq!(245.0, val);
        assert_eq!(3, consumed);
    }

    #[test]
    fn lexer_consume_num_test_2() {
        let digits_2 = vec!['২', '৪', '৫', ' ', '২'];

        let (val, consumed) = consume_num(&digits_2, 0, 1, "test.pakhi").unwrap();
        assert_eq!(245.0, val);
        assert_eq!(3, consumed);
    }

    #[test]
    fn lexer_consume_num_test_3() {
        let digits_3 = vec!['২', '৪', '৫', '.', '২', '৩', '৬'];

        let (val, consumed) = consume_num(&digits_3, 0, 1, "test.pakhi").unwrap();
        assert_eq!(245.236, val);
        assert_eq!(7, consumed);
    }

    #[test]
    fn lexer_consume_num_test_4() {
        let digits_4 = vec!['-', '২', '৪', '৫', '.', '২', '৩', '৬'];

        let (val, consumed) = consume_num(&digits_4, 0, 1, "test.pakhi").unwrap();
        assert_eq!(-245.236, val);
        assert_eq!(8, consumed);
    }

    #[test]
    fn lexer_consume_num_test_5() {
        let digits_5 = vec!['০'];

        let (val, consumed) = consume_num(&digits_5, 0, 1, "test.pakhi").unwrap();
        assert_eq!(0.0, val);
        assert_eq!(1, consumed);
    }

    #[test]
    fn lexer_consume_string_test() {
        let string: Vec<char> = "\" var a = 45;\"".chars().collect();

        let (val, consumed) = consume_string(&string, 0);
        assert_eq!(" var a = 45;", val);
        assert_eq!(14, consumed);
    }

    #[test]
    fn lexer_keyword_test_1() {
        let kword: Vec<char> = "ফাং".chars().collect();
        let t = keyword(&kword, 0, "test.pakhi".to_string()).unwrap();
        assert_eq!(TokenKind::Function, t.kind);
    }

    #[test]
    fn lexer_keyword_test_2() {
        let kword: Vec<char> = "নাম".chars().collect();
        let t = keyword(&kword, 0, "test.pakhi".to_string()).unwrap();
        assert_eq!(TokenKind::Var, t.kind);
    }

    #[test]
    fn lexer_keyword_test_3() {
        let kword: Vec<char> = "লুপ".chars().collect();
        let t = keyword(&kword, 0, "test.pakhi".to_string()).unwrap();
        assert_eq!(TokenKind::Loop, t.kind);
    }

    #[test]
    fn lexer_keyword_test_4() {
        let kword: Vec<char> = "abc".chars().collect();
        assert!(keyword(&kword, 0, "test.pakhi".to_string()).is_none());
    }
}
