use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: Vec<char>,
    pub line: u32,
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
    PrintNoEOL,
    EOT, // represents end of token, only needed for parsing to indicate
         // all previous tokens were consumed
}

pub fn tokenize(src: Vec<char>) -> Vec<Token> {
    let mut current_i = 0;
    let mut line = 1;

    let mut tokens: Vec<Token> = Vec::new();

    while current_i < src.len() {
        // c represents total chars consumed by token t
        // l represents total line consumed by token t
        let (t, c, l) = consume(&src, current_i, line);
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
    });

    tokens
}

fn consume(src: &Vec<char>, start: usize, line: u32) -> (Option<Token>, usize, u32) {
    let consumed_char: usize;
    let consumed_line: u32;
    let token: Token;

    match src[start] {
        '-'|'০'|'১'|'২'|'৩'|'৪'|'৫'|'৬'|'৭'|'৮'|'৯' => {
            if src[start+1].is_numeric() || src[start].is_numeric() {
                // negative number, unary '-' operator
                let (val, consumed) = consume_num(src, start);

                consumed_char = consumed;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::Num(val),
                    lexeme: src[start..(start+consumed_char)].to_vec(),
                    line: line + consumed_line,
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
                    }
                } else {
                    // binary '-' operator
                    consumed_char = 1;
                    consumed_line = 0;
                    token = Token {
                        kind: TokenKind::Minus,
                        lexeme: src[start..(start+1)].to_vec(),
                        line,
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
            }
        },
        '*' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Multiply,
                lexeme: src[start..(start+1)].to_vec(),
                line,
            }
        },
        '/' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Division,
                lexeme: src[start..(start+1)].to_vec(),
                line,
            }
        },
        '%' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Remainder,
                lexeme: src[start..(start+1)].to_vec(),
                line,
            }
        },
        '&' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::And,
                lexeme: src[start..(start+1)].to_vec(),
                line,
            }
        },
        '|' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Or,
                lexeme: src[start..(start+1)].to_vec(),
                line,
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
                }
            } else {
                consumed_char = 1;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::Not,
                    lexeme: src[start..(start+1)].to_vec(),
                    line,
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
            }
        },
        '#' => {
            let (char_skipped, lines_skipped) = skip_comment_block(src, start);
            consumed_char = char_skipped;
            consumed_line = lines_skipped;
            token = Token {
                kind: TokenKind::Comment,
                lexeme: src[start..(start+char_skipped)].to_vec(),
                line,
            }
        },
        ';' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Semicolon,
                lexeme: src[start..(start+1)].to_vec(),
                line,
            }
        },
        ',' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::Comma,
                lexeme: src[start..(start+1)].to_vec(),
                line,
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
            }
        },
        '(' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::ParenStart,
                lexeme: src[start..(start+1)].to_vec(),
                line,
            }
        },
        ')' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::ParenEnd,
                lexeme: src[start..(start+1)].to_vec(),
                line,
            }
        },
        '{' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::CurlyBraceStart,
                lexeme: src[start..(start+1)].to_vec(),
                line,
            }
        },
        '}' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::CurlyBraceEnd,
                lexeme: src[start..(start+1)].to_vec(),
                line,
            }
        },
        '[' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::SquareBraceStart,
                lexeme: src[start..(start+1)].to_vec(),
                line,
            }
        },
        ']' => {
            consumed_char = 1;
            consumed_line = 0;
            token = Token {
                kind: TokenKind::SquareBraceEnd,
                lexeme: src[start..(start+1)].to_vec(),
                line,
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
                }
            } else {
                consumed_char = 1;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::Equal,
                    lexeme: src[start..(start+1)].to_vec(),
                    line,
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
                }
            } else {
                consumed_char = 1;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::LessThan,
                    lexeme: src[start..(start+1)].to_vec(),
                    line,
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
                }
            } else {
                consumed_char = 1;
                consumed_line = 0;
                token = Token {
                    kind: TokenKind::GreaterThan,
                    lexeme: src[start..(start+1)].to_vec(),
                    line,
                }
            }
        },
        ' '|'\r' => {
            consumed_char = 1;
            consumed_line = 0;
            return (None, consumed_char, consumed_line);
        },
        '\n' => {
            consumed_char = 1;
            consumed_line = 1;
            return (None, consumed_char, consumed_line);
        },
        _ => {
            // if nothing matches must be an identifier
            let (t, consumed) = consume_identifier(src, start, line);

            consumed_char = consumed;
            consumed_line = 0;
            token = t;
        },
    }

    (Some(token), consumed_char, consumed_line)
}

fn consume_num(src: &Vec<char>, start: usize) -> (f64, usize) {
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
                panic!("Number is not properly formatted");
            }
            in_fractional_part = true;
            consumed += 1;
            i += 1;
            continue;
        }

        if in_fractional_part {
            fractional_val = (fractional_val * 10.0) + bn_digit_to_en_digit(src[i]);
            consumed += 1;
            i += 1;
        } else {
            val = (val * 10.0) + bn_digit_to_en_digit(src[i]);
            consumed += 1;
            i += 1;
        }
    }
    fractional_val = fractional_val / (10_f64.powf(fractional_val.to_string().len() as f64));

    if is_negative {
        ((val + fractional_val) * -1.0, consumed)
    } else {
        ((val + fractional_val), consumed)
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

fn consume_identifier(src: &Vec<char>, start: usize, line: u32) -> (Token, usize) {
    let mut consumed = 0;
    let mut char_vec: Vec<char>= Vec::new();

    let mut i = start;
    while i < src.len() && is_valid_identifier_char(src[i]) {
        char_vec.push(src[i]);
        consumed += 1;
        i +=1;
    }

    let token = match keyword(&char_vec, line) {
        Some(t) => t,
        None => Token {
            kind: TokenKind::Identifier,
            lexeme: src[start..(start+consumed)].to_vec(),
            line,
        }
    };

    (token, consumed)
}

fn is_valid_identifier_char(c: char) -> bool {
    if c == '-' || c == '_' {
        return true;
    }
    !c.is_ascii_whitespace() && !c.is_ascii_punctuation() && !c.is_ascii_control()
}

fn keyword(char_vec: &Vec<char>, line: u32) -> Option<Token> {
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

    match keyword_map.remove(char_vec) {
        Some(token_kind) => Some(Token {
            kind: token_kind,
            lexeme: char_vec.to_vec(),
            line,
        }),
        None => None,
    }
}

fn bn_digit_to_en_digit(digit: char) -> f64 {
    match digit {
        '০' => 0.0,
        '১' => 1.0,
        '২' => 2.0,
        '৩' => 3.0,
        '৪' => 4.0,
        '৫' => 5.0,
        '৬' => 6.0,
        '৭' => 7.0,
        '৮' => 8.0,
        '৯' => 9.0,
        _ => panic!(),
    }
}

fn skip_comment_block(src: &Vec<char>, start: usize) -> (usize, u32) {
    let mut char_skipped: usize = 1;
    let mut lines_skipped: u32 = 0;
    while src[start + char_skipped] != '#' {
        if (start + char_skipped + 1) > src.len() - 1 {
            panic!("Comment block wasn't closed");
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

    (char_skipped, lines_skipped)
}

#[cfg(test)]
mod tests {
    use crate::frontend::lexer::*;

    #[test]
    fn lexer_consume_num_test_1() {
        let digits_1 = vec!['২', '৪', '৫'];

        let (val, consumed) = consume_num(&digits_1, 0);
        assert_eq!(245.0, val);
        assert_eq!(3, consumed);
    }

    #[test]
    fn lexer_consume_num_test_2() {
        let digits_2 = vec!['২', '৪', '৫', ' ', '২'];

        let (val, consumed) = consume_num(&digits_2, 0);
        assert_eq!(245.0, val);
        assert_eq!(3, consumed);
    }

    #[test]
    fn lexer_consume_num_test_3() {
        let digits_3 = vec!['২', '৪', '৫', '.', '২', '৩', '৬'];

        let (val, consumed) = consume_num(&digits_3, 0);
        assert_eq!(245.236, val);
        assert_eq!(7, consumed);
    }

    #[test]
    fn lexer_consume_num_test_4() {
        let digits_4 = vec!['-', '২', '৪', '৫', '.', '২', '৩', '৬'];

        let (val, consumed) = consume_num(&digits_4, 0);
        assert_eq!(-245.236, val);
        assert_eq!(8, consumed);
    }

    #[test]
    fn lexer_consume_num_test_5() {
        let digits_5 = vec!['০'];

        let (val, consumed) = consume_num(&digits_5, 0);
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
        let t = keyword(&kword, 0).unwrap();
        assert_eq!(TokenKind::Function, t.kind);
    }

    #[test]
    fn lexer_keyword_test_2() {
        let kword: Vec<char> = "নাম".chars().collect();
        let t = keyword(&kword, 0).unwrap();
        assert_eq!(TokenKind::Var, t.kind);
    }

    #[test]
    fn lexer_keyword_test_3() {
        let kword: Vec<char> = "লুপ".chars().collect();
        let t = keyword(&kword, 0).unwrap();
        assert_eq!(TokenKind::Loop, t.kind);
    }

    #[test]
    fn lexer_keyword_test_4() {
        let kword: Vec<char> = "abc".chars().collect();
        assert!(keyword(&kword, 0).is_none());
    }

    #[test]
    fn lexer_var_declare() {
        let tokens = tokenize("নাম ল = ০;".chars().collect::<Vec<char>>());
        assert_eq!(TokenKind::Var, tokens[0].kind);
        assert_eq!(TokenKind::Identifier, tokens[1].kind);
        assert_eq!(TokenKind::Equal, tokens[2].kind);
        assert_eq!(TokenKind::Num(0.0), tokens[3].kind);
        assert_eq!(TokenKind::Semicolon, tokens[4].kind);
    }

    #[test]
    fn lexer_nameless_record_literal() {
        let tokens = tokenize(
            r#"@ {"key" -> ১,}"#.chars().collect::<Vec<char>>());
        assert_eq!(TokenKind::At, tokens[0].kind);
        assert_eq!(TokenKind::CurlyBraceStart, tokens[1].kind);
        assert_eq!(TokenKind::String(String::from("key")), tokens[2].kind);
        assert_eq!(TokenKind::Map, tokens[3].kind);
        assert_eq!(TokenKind::Num(1.0), tokens[4].kind);
        assert_eq!(TokenKind::Comma, tokens[5].kind);
        assert_eq!(TokenKind::CurlyBraceEnd, tokens[6].kind);
    }

    #[test]
    fn lexer_comment_block() {
        let tokens = tokenize("# this is a comment # \
                                                নাম ল = ০;\
                                              #this is a second comment#".chars().collect::<Vec<char>>());
        assert_eq!(TokenKind::Comment, tokens[0].kind);
        assert_eq!(TokenKind::Var, tokens[1].kind);
        assert_eq!(TokenKind::Identifier, tokens[2].kind);
        assert_eq!(TokenKind::Equal, tokens[3].kind);
        assert_eq!(TokenKind::Num(0.0), tokens[4].kind);
        assert_eq!(TokenKind::Semicolon, tokens[5].kind);
        assert_eq!(TokenKind::Comment, tokens[6].kind);
    }
}