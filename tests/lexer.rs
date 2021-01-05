use pakhi::frontend::lexer::{tokenize, TokenKind};

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