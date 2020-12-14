use crate::frontend::lexer::Token;
use crate::frontend::lexer::TokenKind;

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Print(Expr),
    PrintNoEOL(Expr),
    Assignment(Assignment),
    Expression(Expr),
    BlockStart,
    BlockEnd,
    FuncDef,
    Return(Expr),
    If(Expr),
    Loop,
    Continue,
    Break,
    Else,
    EOS,    // represents end of statement, only needed for interpreting to indicate
            // all previous statements were consumed
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment {
    pub kind: AssignmentKind,
    pub var_name: Token,
    // assignment could me made to array element, so indexes are needed
    pub indexes: Vec<Expr>,
    pub init_value: Option<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssignmentKind {
    FirstAssignment,
    Reassignment,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    ArrayIndexing(Box<Expr>, Box<Expr>),
    Or(Or),
    And(And),
    Equality(Binary),
    Comparison(Binary),
    AddOrSub(Binary),
    MulOrDivOrRemainder(Binary),
    Unary(Unary),
    Call(FunctionCall),
    Primary(Primary),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Primary {
    Nil,
    Bool(bool),
    Num(f64),
    String(String),
    List(Vec<Expr>),
    NamelessRecord((Vec<Expr>, Vec<Expr>)),
    Var(Token),
    Group(Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Or {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct And {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Binary {
    pub operator: TokenKind,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Unary {
    pub operator: TokenKind,
    pub right: Box<Expr>
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub expr: Box<Expr>,
    pub arguments: Vec<Expr>,
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
        }
    }

    fn parse(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();

        while self.tokens[self.current].kind != TokenKind::EOT {
            statements.push(self.statements());

            if self.tokens[self.current].kind == TokenKind::Semicolon {
                // useful semicolon should be consumed by self.statements()
                // if not consumed assuming not useful semicolon
                // function call needs this
                // skipping semicolon
                self.current += 1;
                continue;
            }
        }
        statements.push(Stmt::EOS);

        statements
    }

    fn statements(&mut self) -> Stmt {
        match self.tokens[self.current].kind {
            TokenKind::Print => self.print_stmt(),
            TokenKind::PrintNoEOL => {
                // consuming token
                self.current += 1;
                let expr = self.expression();
                //consuming last ';' of print statement
                self.current += 1;

                Stmt::PrintNoEOL(expr)
            },
            TokenKind::Var => self.assignment_stmt(),
            TokenKind::Identifier => {
                // probably array indexing after function call won't work
                if self.tokens[self.current + 1].kind == TokenKind::ParenStart {
                    // assuming its a function call statement
                    let expr = self.expression();

                    return Stmt::Expression(expr);
                }
                // if next token is not paren assuming it's a reassignment statement, or expression
                // statement which ony has identifier
                self.re_assignment_stmt()
            },
            TokenKind::CurlyBraceStart => self.block_start(),
            TokenKind::CurlyBraceEnd => self.block_end(),
            TokenKind::If => self.if_statement(),
            TokenKind::Else => self.else_statement(),
            TokenKind::Loop => self.loop_stmt(),
            TokenKind::Continue => self.continue_stmt(),
            TokenKind::Break => self.break_stmt(),
            TokenKind::Function => self.func_def_stmt(),
            TokenKind::Return => self.return_stmt(),
            TokenKind::At => todo!(),
             _ => panic!("Err at line: {}\nDebug token{:#?}",
                        self.tokens[self.current].line, self.tokens[self.current]),
        }
    }

    fn assignment_stmt(&mut self) -> Stmt {
        // consuming var token
        self.current += 1;
        if self.tokens[self.current].kind != TokenKind::Identifier {
            panic!("Error at line: {}\nExpected a Identifier", self.tokens[self.current].line);
        }

        let var_name = self.tokens[self.current].clone();

        // consuming identifier and '=' token
        self.current += 2;

        let stmt;
        if self.tokens[self.current].kind == TokenKind::Semicolon {
            // no value provided to initialize variable
            stmt = Stmt::Assignment(Assignment {
                kind: AssignmentKind::FirstAssignment,
                var_name,
                indexes: Vec::new(),
                init_value: None,
            });
        } else {
            let expr = self.expression();
            // init value provided for assigning to variable
            stmt = Stmt::Assignment(Assignment {
                kind: AssignmentKind::FirstAssignment,
                var_name,
                indexes: Vec::new(),
                init_value: Some(expr),
            });
        }

        if self.tokens[self.current].kind != TokenKind::Semicolon {
            panic!("Error at line: {}\nExpected a ;", self.tokens[self.current].line);
        }
        // consuming ; token
        self.current += 1;

        stmt
    }

    fn re_assignment_stmt(&mut self) -> Stmt {
        if self.tokens[self.current+1].kind != TokenKind::Equal &&
            self.tokens[self.current+1].kind != TokenKind::SquareBraceStart {
            // not a reassignment, only expression statement;
            return self.expression_stmt();
        }

        let var_name = self.tokens[self.current].clone();
        // consuming Identifier token
        self.current += 1;

        // indexes will be populated only if assigning to array element, otherwise it will be empty
        let mut indexes: Vec<Expr> = Vec::new();
        while self.tokens[self.current].kind != TokenKind::Equal {
            let index = self.expression();
            if let Expr::Primary(Primary::List(_)) = index {
                indexes.push(index);
            } else {
                panic!("Error at line {}. Array index expected", self.tokens[self.current].line);
            }
        }

        if self.tokens[self.current].kind != TokenKind::Equal {
           panic!("Expected '=' at line {}", self.tokens[self.current].line);
        }
        // consuming '=' token
        self.current += 1;

        let expr = self.expression();

        // consuming ; token
        self.current += 1;

        let stmt = Stmt::Assignment(Assignment {
            kind: AssignmentKind::Reassignment,
            var_name,
            indexes,
            init_value: Some(expr),
        });

        stmt
    }

    // expression need to be wrapped in expression stmt because interpreter only accepts vec of stmts
    fn expression_stmt(&mut self) -> Stmt {
        Stmt::Expression(self.expression())
    }

    fn print_stmt(&mut self) -> Stmt {
        // consuming print token
        self.current += 1;
        let expr = self.expression();
        //consuming last ';' of print statement
        self.current += 1;

        Stmt::Print(expr)
    }

    fn func_def_stmt(&mut self) -> Stmt {
        // consuming function token
        self.current += 1;

        Stmt::FuncDef
    }

    fn return_stmt(&mut self) -> Stmt {
        // consuming return token
        self.current += 1;

        let mut return_value = Expr::Primary(Primary::Nil);
        if self.tokens[self.current].kind != TokenKind::Semicolon {
            // if not semicolon function return a value
            return_value = self.expression();
        }

        //consuming ; token
        self.current += 1;
        Stmt::Return(return_value)
    }

    fn block_start(&mut self) -> Stmt {
        // consuming { token
        self.current += 1;

        Stmt::BlockStart
    }

    fn block_end(&mut self) -> Stmt {
        // consuming } token
        self.current += 1;

        Stmt::BlockEnd
    }

    fn loop_stmt(&mut self) -> Stmt {
        // consuming loop token
        self.current += 1;

        Stmt::Loop
    }

    fn continue_stmt(&mut self) -> Stmt {
        // consuming loop token
        self.current += 2;

        Stmt::Continue
    }

    fn break_stmt(&mut self) -> Stmt {
        // consuming break and ; token
        self.current += 2;

        Stmt::Break
    }

    fn if_statement(&mut self) -> Stmt {
        //consuming if token
        self.current += 1;

        let condition = self.expression();

        Stmt::If(condition)
    }

    fn else_statement(&mut self) -> Stmt {
        //consuming else token
        self.current += 1;

        Stmt::Else
    }

    fn expression(&mut self) -> Expr {
       self.or()
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();

        while self.tokens[self.current].kind == TokenKind::Or {
            self.current += 1;
            let right = self.and();
            expr = Expr::Or(Or {
                left: Box::new(expr),
                right: Box::new(right),
            })
        }

        expr
    }

    fn and(&mut self) -> Expr {
        let mut expr = self.equality();

        while self.tokens[self.current].kind == TokenKind::And {
            self.current += 1;
            let right = self.equality();
            expr = Expr::And(And {
                left: Box::new(expr),
                right: Box::new(right),
            })
        }

        expr
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.tokens[self.current].kind == TokenKind::NotEqual ||
            self.tokens[self.current].kind == TokenKind:: EqualEqual
        {
            let operator = self.tokens[self.current].kind.clone();
            self.current += 1;
            let right = self.comparison();
            expr = Expr::Equality(Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            })
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.addition();

        while self.tokens[self.current].kind == TokenKind::GreaterThan ||
            self.tokens[self.current].kind == TokenKind::GreaterThanOrEqual ||
            self.tokens[self.current].kind == TokenKind::LessThan ||
            self.tokens[self.current].kind == TokenKind::LessThanOrEqual
        {
            let operator = self.tokens[self.current].kind.clone();
            self.current += 1;
            let right = self.addition();
            expr = Expr::Comparison(Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            })
        }

        expr
    }

    fn addition(&mut self) -> Expr {
        let mut expr = self.multiplication();

        while self.tokens[self.current].kind == TokenKind::Plus ||
            self.tokens[self.current].kind == TokenKind::Minus
        {
            let operator = self.tokens[self.current].kind.clone();
            self.current += 1;
            let right = self.multiplication();
            expr = Expr::AddOrSub(Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            })
        }

        expr
    }

    fn multiplication(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.tokens[self.current].kind == TokenKind::Multiply ||
            self.tokens[self.current].kind == TokenKind::Division ||
            self.tokens[self.current].kind == TokenKind::Remainder
        {
            let operator = self.tokens[self.current].kind.clone();
            self.current += 1;
            let right = self.unary();
            expr = Expr::MulOrDivOrRemainder(Binary {
                left: Box::new(expr),
                right: Box::new(right),
                operator,
            })
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.tokens[self.current].kind == TokenKind::Not ||
            self.tokens[self.current].kind == TokenKind::Minus
        {
            let operator = self.tokens[self.current].kind.clone();
            self.current += 1;
            let right = self.unary();
            let expr = Expr::Unary(Unary {
                operator,
                right: Box::new(right),
            });

            return expr;
        }
        return self.call()
    }

    fn finish_call(&mut self, calle: Expr) -> Expr {
        let mut arguments: Vec<Expr> = Vec::new();

        if self.tokens[self.current].kind != TokenKind::ParenEnd {
            loop {
                arguments.push(self.expression());
                if self.tokens[self.current].kind == TokenKind::Comma {
                    // consuming , token
                    self.current += 1;
                } else {
                    // no comma means all arguments consumed, so breaking out of
                    // arguments consuming loop
                    break;
                }
            }
        }

        //consuming parenEnd
        self.current += 1;

        Expr::Call(FunctionCall {
            expr: Box::new(calle),
            arguments,
        })}

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();

        // rewrite this to handle method invocation
        loop {
            if self.tokens[self.current].kind == TokenKind::ParenStart {
                self.current += 1;
                expr = self.finish_call(expr);
            } else {
                break;
            }
        }

        expr
    }

    fn primary(&mut self) -> Expr {
        match self.tokens[self.current].kind.clone() {
            TokenKind::Bool(b) => {
                self.current += 1;
                return Expr::Primary(Primary::Bool(b));
            },
            TokenKind::Num(n) => {
                self.current += 1;
                return Expr::Primary(Primary::Num(n));
            },
            TokenKind::String(s) => {
                self.current += 1;
                return Expr::Primary(Primary::String(s));
            },
            TokenKind::Identifier => {
                // this is identifier or array index expression

                let mut expr = Expr::Primary(Primary::Var(self.tokens[self.current].clone()));
                // consuming identifier token
                self.current += 1;

                while self.tokens[self.current].kind == TokenKind::SquareBraceStart {
                    // consuming [ token
                    self.current += 1;
                    let i = self.expression();
                    if self.tokens[self.current].kind != TokenKind::SquareBraceEnd {
                        panic!("Expected ] at line {}", self.tokens[self.current].line);
                    }
                    // consuming ] token
                    self.current += 1;

                    expr = Expr::ArrayIndexing(Box::new(expr), Box::new(i));
                }

                return expr;
            },
            TokenKind::ParenStart => {
                self.current += 1;
                let expr = self.expression();
                // consuming parenEnd ')'
                self.current += 1;
                return  Expr::Primary(Primary::Group(Box::new(expr)));
            },
            TokenKind::SquareBraceStart => {
                // consuming [ Token
                self.current += 1;

                let mut array_literal: Vec<Expr> = Vec::new();

                while self.tokens[self.current].kind != TokenKind::SquareBraceEnd {
                    array_literal.push(self.expression());

                    if self.tokens[self.current].kind == TokenKind::Comma {
                        //consuming comma token
                        self.current += 1;
                    }
                }

                if self.tokens[self.current].kind != TokenKind::SquareBraceEnd {
                    panic!("Expecting ] at line: {}", self.tokens[self.current].line);
                }
                //consuming ] Token
                self.current += 1;

                return Expr::Primary(Primary::List(array_literal));
            },
            TokenKind::At => {
                // Iterates through all key-value pair and saves them in different vec. Then returns
                // vec of key-value pair as tuples

                // consuming @ token
                self.current += 1;

                if self.tokens[self.current].kind != TokenKind::CurlyBraceStart {
                    panic!("Expected {{ after @ at line: {}", self.tokens[self.current].line);
                }
                // consuming { token
                self.current += 1;

                let mut keys: Vec<Expr>  = Vec::new();
                let mut values: Vec<Expr>  = Vec::new();

                while self.tokens[self.current].kind != TokenKind::CurlyBraceEnd {
                    // pushing key of a key-value pair
                    keys.push(self.expression());

                    // Token after key should be colon
                    if self.tokens[self.current].kind != TokenKind::Colon {
                        panic!("Expected : after key name at line: {}", self.tokens[self.current].line);
                    }
                    // consuming : token
                    self.current += 1;

                    // pushing value of a key-value pair
                    values.push(self.expression());

                    if self.tokens[self.current].kind == TokenKind::Comma {
                        // consuming , token
                        self.current += 1
                    }
                }

                if self.tokens[self.current].kind != TokenKind::CurlyBraceEnd {
                    panic!("Expecting }} at line: {}", self.tokens[self.current].line);
                }
                //consuming } Token
                self.current += 1;

                return Expr::Primary(Primary::NamelessRecord((keys, values)));
            },
            _ => panic!("Error at line: {}\n Debug Token: {:#?}",
                        self.tokens[self.current].line, self.tokens[self.current]),
        }
    }
}

// --------------Entry-pint--------------------
pub fn parse(tokens: Vec<Token>) -> Vec<Stmt> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use crate::frontend::{lexer, parser};
    use crate::frontend::parser::*;
    use crate::parser::AssignmentKind::FirstAssignment;
    use crate::parser::Primary::{NamelessRecord, Num};
    use crate::lexer::TokenKind::{Identifier, Plus};
    use crate::parser::Expr::AddOrSub;

    #[test]
    fn parse_test_primary_num() {
        let tokens = lexer::tokenize("দেখাও ৫৩.৬;".chars().collect());
        let ast = parse(tokens);
        let expected_ast = Stmt::Print(Expr::Primary(Primary::Num(53.6)));
        assert_eq!(expected_ast, ast[0]);
    }

    #[test]
    fn parse_test_binary_addition() {
        let tokens = lexer::tokenize("দেখাও -৫৩.৬ + ৬;".chars().collect());
        let ast = parse(tokens);
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
        let ast = parse(tokens);
        let expected_ast = Stmt::Print(Expr::Primary(Primary::String(String::from("this is a test"))));
        assert_eq!(expected_ast, ast[0]);
    }

    #[test]
    fn parse_test_print_expr() {
        let tokens = lexer::tokenize("দেখাও ১ + ৩ * ২;".chars().collect());
        let ast = parse(tokens);
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
        let ast = parse(tokens);
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
        let ast = parse(tokens);
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
        let ast = parse(tokens);
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
        let ast = parse(tokens);
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
        let ast = parse(tokens);
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
        let ast = parse(tokens);
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
        let ast = parse(tokens);
        let expected_ast = Stmt::Print(Expr::And(And {
            left: Box::from(Expr::Primary(Primary::Bool(true))),
            right: Box::from(Expr::Primary(Primary::Bool(false))),
        }));

        assert_eq!(expected_ast, ast[0]);
    }

    #[test]
    fn parse_test_print_logical_or() {
        let tokens = lexer::tokenize("দেখাও সত্য | মিথ্যা;".chars().collect());
        let ast = parse(tokens);
        let expected_ast = Stmt::Print(Expr::Or(Or {
            left: Box::from(Expr::Primary(Primary::Bool(true))),
            right: Box::from(Expr::Primary(Primary::Bool(false))),
        }));

        assert_eq!(expected_ast, ast[0]);
    }

    #[test]
    fn parse_test_print_logical_not() {
        let tokens = lexer::tokenize("দেখাও !সত্য;".chars().collect());
        let ast = parse(tokens);
        let expected_ast = Stmt::Print(Expr::Unary(Unary {
            operator: TokenKind::Not,
            right: Box::from(Expr::Primary(Primary::Bool(true))),
        }));

        assert_eq!(expected_ast, ast[0]);
    }

    #[test]
    fn parse_test_assignment_string() {
        let tokens = lexer::tokenize("নাম ল = \"red\";".chars().collect());
        let ast = parse(tokens);
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
        let ast = parse(tokens);
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
                                                                "key": ১,
                                                                "key_2": "string",
                                                                "key": ১ + ১,
                                                            };"#.chars().collect());
        let ast = parse(tokens);
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
}