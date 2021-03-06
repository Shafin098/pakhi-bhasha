use pakhi::frontend::{lexer, parser};
use pakhi::frontend::parser::{Stmt, Primary, Expr, Binary, Unary, Assignment, AssignmentKind, And, Or, parse};
use pakhi::frontend::lexer::{TokenKind, Token};
use pakhi::frontend::parser::AssignmentKind::FirstAssignment;
use pakhi::frontend::lexer::TokenKind::{Identifier, Plus};
use pakhi::frontend::parser::Primary::{NamelessRecord, Num};
use pakhi::frontend::parser::Expr::AddOrSub;

#[test]
fn parse_test_primary_num() {
    let tokens = lexer::tokenize("দেখাও ৫৩.৬;".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::Primary(Primary::Num(53.6), 1, "test.pakhi".to_string())
                                           , 1, "test.pakhi".to_string());
            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_binary_addition() {
    let tokens = lexer::tokenize("দেখাও -৫৩.৬ + ৬;".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::AddOrSub(Binary {
                operator: TokenKind::Plus,
                left: Box::new(Expr::Primary(Primary::Num(-53.6), 1, "test.pakhi".to_string())),
                right: Box::new(Expr::Primary(Primary::Num(6.0), 1, "test.pakhi".to_string())),
            }, 1, "test.pakhi".to_string()), 1, "test.pakhi".to_string());
            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_primary_string() {
    let tokens = lexer::tokenize("দেখাও \"this is a test\";".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::Primary(Primary::String(String::from("this is a test")),
                                                         1, "test.pakhi".to_string()), 1, "test.pakhi".to_string());
            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_print_expr() {
    let tokens = lexer::tokenize("দেখাও ১ + ৩ * ২;".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::AddOrSub(Binary {
                operator: TokenKind::Plus,
                left: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
                right: Box::from(Expr::MulOrDivOrRemainder(Binary {
                    operator: TokenKind::Multiply,
                    left: Box::from(Expr::Primary(Primary::Num(3.0), 1, "test.pakhi".to_string())),
                    right: Box::from(Expr::Primary(Primary::Num(2.0), 1, "test.pakhi".to_string())),
                }, 1, "test.pakhi".to_string()))
            }, 1, "test.pakhi".to_string()), 1, "test.pakhi".to_string());
            assert_eq!(expected_ast, ast[0]);
    },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_print_equality() {
    let tokens = lexer::tokenize("দেখাও ১ == ১;".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::Equality(Binary {
                operator: TokenKind::EqualEqual,
                left: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
                right: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
            }, 1, "test.pakhi".to_string()), 1, "test.pakhi".to_string());

            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_print_not_equal() {
    let tokens = lexer::tokenize("দেখাও ১ != ১;".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::Equality(Binary {
                operator: TokenKind::NotEqual,
                left: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
                right: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
            }, 1, "test.pakhi".to_string()), 1, "test.pakhi".to_string());

            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_print_comparison_less() {
    let tokens = lexer::tokenize("দেখাও ১ < ১;".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::Comparison(Binary {
                operator: TokenKind::LessThan,
                left: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
                right: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
            }, 1, "test.pakhi".to_string()), 1, "test.pakhi".to_string());

            assert_eq!(expected_ast, ast[0]);
        }
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_comaprison_greater() {
    let tokens = lexer::tokenize("দেখাও ১ > ১;".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::Comparison(Binary {
                operator: TokenKind::GreaterThan,
                left: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
                right: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
            }, 1, "test.pakhi".to_string()), 1, "test.pakhi".to_string());

            assert_eq!(expected_ast, ast[0]);
        }
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_comparison_less_or_equal() {
    let tokens = lexer::tokenize("দেখাও ১ <= ১;".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::Comparison(Binary {
                operator: TokenKind::LessThanOrEqual,
                left: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
                right: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
            }, 1, "test.pakhi".to_string()), 1, "test.pakhi".to_string());

            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_comaprison_greater_or_equla() {
    let tokens = lexer::tokenize("দেখাও ১ >= ১;".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::Comparison(Binary {
                operator: TokenKind::GreaterThanOrEqual,
                left: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
                right: Box::from(Expr::Primary(Primary::Num(1.0), 1, "test.pakhi".to_string())),
            }, 1, "test.pakhi".to_string()), 1, "test.pakhi".to_string());

            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_print_logical_and() {
    let tokens = lexer::tokenize("দেখাও সত্য & মিথ্যা;".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::And(And {
                left: Box::from(Expr::Primary(Primary::Bool(true), 1, "test.pakhi".to_string())),
                right: Box::from(Expr::Primary(Primary::Bool(false), 1, "test.pakhi".to_string())),
            }, 1, "test.pakhi".to_string()), 1, "test.pakhi".to_string());

            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_print_logical_or() {
    let tokens = lexer::tokenize("দেখাও সত্য | মিথ্যা;".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::Or(Or {
                left: Box::from(Expr::Primary(Primary::Bool(true), 1, "test.pakhi".to_string())),
                right: Box::from(Expr::Primary(Primary::Bool(false), 1, "test.pakhi".to_string())),
            }, 1, "test.pakhi".to_string()), 1, "test.pakhi".to_string());

            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_print_logical_not() {
    let tokens = lexer::tokenize("দেখাও !সত্য;".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Print(Expr::Unary(Unary {
                operator: TokenKind::Not,
                right: Box::from(Expr::Primary(Primary::Bool(true), 1, "test.pakhi".to_string())),
            }, 1, "test.pakhi".to_string()), 1, "test.pakhi".to_string());

            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_assignment_string() {
    let tokens = lexer::tokenize("নাম ল = \"red\";".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Assignment(Assignment {
                kind: AssignmentKind::FirstAssignment,
                var_name: Token {
                    kind: TokenKind::Identifier,
                    lexeme: vec!['ল'],
                    line: 1,
                    src_file_path: "test.pakhi".to_string(),
                },
                indexes: Vec::new(),
                init_value: Some(Expr::Primary(Primary::String("red".to_string()), 1, "test.pakhi".to_string())),
            }, 1, "test.pakhi".to_string());

            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_re_assignment_string() {
    let tokens = lexer::tokenize("ল = \"red\";".chars().collect(),
                                 "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) => {
            let expected_ast = Stmt::Assignment(Assignment {
                kind: AssignmentKind::Reassignment,
                var_name: Token {
                    kind: TokenKind::Identifier,
                    lexeme: vec!['ল'],
                    line: 1,
                    src_file_path: "test.pakhi".to_string(),
                },
                indexes: Vec::new(),
                init_value: Some(Expr::Primary(Primary::String("red".to_string()), 1, "test.pakhi".to_string())),
            }, 1, "test.pakhi".to_string());

            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}

#[test]
fn parse_test_namesless_record_literal() {
    let tokens = lexer::tokenize(r#"নাম ক =  @{
                                                                "key" -> ১,
                                                                "key_2" -> "string",
                                                                "key" -> ১ + ১,
                                                            };"#.chars().collect(),
                                                        "test.pakhi".to_string()).unwrap();
    let parse_result = parse(String::from("test.pakhi"), tokens);
    match parse_result {
        Ok(ast) =>  {
            let expected_ast = Stmt::Assignment(Assignment {
                kind: FirstAssignment,
                var_name: Token {
                    kind: Identifier,
                    lexeme: vec!['ক'],
                    line: 1,
                    src_file_path: "test.pakhi".to_string(),
                },
                indexes: vec![],
                init_value: Some(
                    parser::Expr::Primary(NamelessRecord(
                        (
                            vec![
                                parser::Expr::Primary(Primary::String("key".to_string()), 2, "test.pakhi".to_string()),
                                parser::Expr::Primary(Primary::String("key_2".to_string()), 3, "test.pakhi".to_string()),
                                parser::Expr::Primary(Primary::String("key".to_string()), 4, "test.pakhi".to_string()),
                            ],
                            vec![
                                parser::Expr::Primary(Num(1.0), 2, "test.pakhi".to_string()),
                                parser::Expr::Primary(Primary::String("string".to_string()), 3, "test.pakhi".to_string()),
                                AddOrSub(Binary {
                                    operator: Plus,
                                    left: Box::from(parser::Expr::Primary(Primary::Num(1.0), 4, "test.pakhi".to_string())),
                                    right: Box::from(parser::Expr::Primary(Primary::Num(1.0), 4, "test.pakhi".to_string())),
                                }, 4, "test.pakhi".to_string()),
                            ],
                        ),
                    ), 1, "test.pakhi".to_string()),
                ),
            }, 1, "test.pakhi".to_string());

            assert_eq!(expected_ast, ast[0]);
        },
        Err(e) => panic!("err: {:?}", e),
    }
}