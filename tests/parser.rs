use pakhi::{lexer, parser};
use pakhi::parser::{parse, Stmt, Primary, Expr, Binary, Unary, Assignment, AssignmentKind, And, Or};
use pakhi::lexer::{TokenKind, Token};
use pakhi::parser::AssignmentKind::FirstAssignment;
use pakhi::lexer::TokenKind::{Identifier, Plus};
use pakhi::parser::Primary::{NamelessRecord, Num};
use pakhi::parser::Expr::AddOrSub;

#[test]
fn parse_test_primary_num() {
    let tokens = lexer::tokenize("দেখাও ৫৩.৬;".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::Primary(Primary::Num(53.6)));
    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_binary_addition() {
    let tokens = lexer::tokenize("দেখাও -৫৩.৬ + ৬;".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::AddOrSub(Binary {
        operator: TokenKind::Plus,
        left: Box::new(Expr::Primary(Primary::Num(-53.6))),
        right: Box::new(Expr::Primary(Primary::Num(6.0))),
    }));
    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_primary_string() {
    let tokens = lexer::tokenize("দেখাও \"this is a test\";".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::Primary(Primary::String(String::from("this is a test"))));
    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_print_expr() {
    let tokens = lexer::tokenize("দেখাও ১ + ৩ * ২;".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::AddOrSub(Binary {
        operator: TokenKind::Plus,
        left: Box::from(Expr::Primary(Primary::Num(1.0))),
        right: Box::from(Expr::MulOrDivOrRemainder(Binary {
            operator: TokenKind::Multiply,
            left: Box::from(Expr::Primary(Primary::Num(3.0))),
            right: Box::from(Expr::Primary(Primary::Num(2.0))),
        }))
    }));
    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_print_equality() {
    let tokens = lexer::tokenize("দেখাও ১ == ১;".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::Equality(Binary {
        operator: TokenKind::EqualEqual,
        left: Box::from(Expr::Primary(Primary::Num(1.0))),
        right: Box::from(Expr::Primary(Primary::Num(1.0))),
    }));

    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_print_not_equal() {
    let tokens = lexer::tokenize("দেখাও ১ != ১;".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::Equality(Binary {
        operator: TokenKind::NotEqual,
        left: Box::from(Expr::Primary(Primary::Num(1.0))),
        right: Box::from(Expr::Primary(Primary::Num(1.0))),
    }));

    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_print_comparison_less() {
    let tokens = lexer::tokenize("দেখাও ১ < ১;".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::Comparison(Binary {
        operator: TokenKind::LessThan,
        left: Box::from(Expr::Primary(Primary::Num(1.0))),
        right: Box::from(Expr::Primary(Primary::Num(1.0))),
    }));

    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_comaprison_greater() {
    let tokens = lexer::tokenize("দেখাও ১ > ১;".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::Comparison(Binary {
        operator: TokenKind::GreaterThan,
        left: Box::from(Expr::Primary(Primary::Num(1.0))),
        right: Box::from(Expr::Primary(Primary::Num(1.0))),
    }));

    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_comparison_less_or_equla() {
    let tokens = lexer::tokenize("দেখাও ১ <= ১;".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::Comparison(Binary {
        operator: TokenKind::LessThanOrEqual,
        left: Box::from(Expr::Primary(Primary::Num(1.0))),
        right: Box::from(Expr::Primary(Primary::Num(1.0))),
    }));

    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_comaprison_greater_or_equla() {
    let tokens = lexer::tokenize("দেখাও ১ >= ১;".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::Comparison(Binary {
        operator: TokenKind::GreaterThanOrEqual,
        left: Box::from(Expr::Primary(Primary::Num(1.0))),
        right: Box::from(Expr::Primary(Primary::Num(1.0))),
    }));

    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_print_logical_and() {
    let tokens = lexer::tokenize("দেখাও সত্য & মিথ্যা;".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::And(And {
        left: Box::from(Expr::Primary(Primary::Bool(true))),
        right: Box::from(Expr::Primary(Primary::Bool(false))),
    }));

    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_print_logical_or() {
    let tokens = lexer::tokenize("দেখাও সত্য | মিথ্যা;".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::Or(Or {
        left: Box::from(Expr::Primary(Primary::Bool(true))),
        right: Box::from(Expr::Primary(Primary::Bool(false))),
    }));

    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_print_logical_not() {
    let tokens = lexer::tokenize("দেখাও !সত্য;".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Print(Expr::Unary(Unary {
        operator: TokenKind::Not,
        right: Box::from(Expr::Primary(Primary::Bool(true))),
    }));

    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_assignment_string() {
    let tokens = lexer::tokenize("নাম ল = \"red\";".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Assignment(Assignment {
        kind: AssignmentKind::FirstAssignment,
        var_name: Token {
            kind: TokenKind::Identifier,
            lexeme: vec!['ল'],
            line: 1,
        },
        indexes: Vec::new(),
        init_value: Some(Expr::Primary(Primary::String("red".to_string()))),
    });

    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_re_assignment_string() {
    let tokens = lexer::tokenize("ল = \"red\";".chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Assignment(Assignment {
        kind: AssignmentKind::Reassignment,
        var_name: Token {
            kind: TokenKind::Identifier,
            lexeme: vec!['ল'],
            line: 1,
        },
        indexes: Vec::new(),
        init_value: Some(Expr::Primary(Primary::String("red".to_string()))),
    });

    assert_eq!(expected_ast, ast[0]);
}

#[test]
fn parse_test_namesless_record_literal() {
    let tokens = lexer::tokenize(r#"নাম ক =  @{
                                                                "key" -> ১,
                                                                "key_2" -> "string",
                                                                "key" -> ১ + ১,
                                                            };"#.chars().collect());
    let ast = parse(String::from(""), tokens);
    let expected_ast = Stmt::Assignment(
        Assignment {
            kind: FirstAssignment,
            var_name: Token {
                kind: Identifier,
                lexeme: vec!['ক'],
                line: 1,
            },
            indexes: vec![],
            init_value: Some(
                parser::Expr::Primary(
                    NamelessRecord(
                        (
                            vec![
                                parser::Expr::Primary(Primary::String("key".to_string())),
                                parser::Expr::Primary(Primary::String("key_2".to_string())),
                                parser::Expr::Primary(Primary::String("key".to_string())),
                            ],
                            vec![
                                parser::Expr::Primary(Num(1.0)),
                                parser::Expr::Primary(Primary::String("string".to_string())),
                                AddOrSub(
                                    Binary {
                                        operator: Plus,
                                        left: Box::from(parser::Expr::Primary(Primary::Num(1.0))),
                                        right: Box::from(parser::Expr::Primary(Primary::Num(1.0))),
                                    },
                                ),
                            ],
                        ),
                    ),
                ),
            ),
        },
    );

    assert_eq!(expected_ast, ast[0]);
}