use crate::frontend::lexer::Token;
use crate::frontend::lexer::TokenKind;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Print(Expr),
    Assignment,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Or(Or),
    And(And),
    Equality(Binary),
    Comparison(Binary),
    Addition(Binary),
    Multiplication(Binary),
    Unary(Unary),
    Call(FunctionCall),
    Primary(Primary),
}

#[derive(Debug, PartialEq)]
pub enum Primary {
    Bool(bool),
    Num(f64),
    String(String),
    Var(Token),
    Group(Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub struct Or {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq)]
pub struct And {
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq)]
pub struct Binary {
    pub operator: TokenKind,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq)]
pub struct Unary {
    pub operator: TokenKind,
    pub right: Box<Expr>
}

#[derive(Debug, PartialEq)]
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
        }

        statements
    }

    fn statements(&mut self) -> Stmt {
        match self.tokens[self.current].kind {
            TokenKind::Print => self.print_stmt(),
            _ => panic!("Err at line: {}", self.tokens[self.current].line),
        }
    }

    fn print_stmt(&mut self) -> Stmt {
        self.current += 1;
        let expr = self.expression();
        //consuming last ';' of print statement
        self.current += 1;

        Stmt::Print(expr)
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
            expr = Expr::Addition(Binary {
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
            self.tokens[self.current].kind == TokenKind::Division
        {
            let operator = self.tokens[self.current].kind.clone();
            self.current += 1;
            let right = self.unary();
            expr = Expr::Multiplication(Binary {
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
                if self.tokens[self.current].kind != TokenKind::Comma {
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
                self.current += 1;
                return Expr::Primary(Primary::Var(self.tokens[self.current-1].clone()));
            },
            TokenKind::ParenStart => {
                self.current += 1;
                let expr = self.expression();
                // consuming parenEnd '}'
                self.current += 1;
                return  Expr::Primary(Primary::Group(Box::new(expr)));
            },
            _ => panic!("Error at line: {}", self.tokens[self.current].line),
        }
    }
}

// --------------------------------------------
pub fn parse(tokens: Vec<Token>) -> Vec<Stmt> {
    let mut p = Parser::new(tokens);
    p.parse()
}

#[cfg(test)]
mod tests {
    use crate::frontend::lexer;
    use crate::frontend::parser::*;

    #[test]
    fn parse_test_1() {
        let tokens = lexer::tokenize("দেখাও ৫৩.৬;".chars().collect());
        let ast = parse(tokens);
        let expected_ast = Stmt::Print(Expr::Primary(Primary::Num(53.6)));
        assert_eq!(expected_ast, ast[0]);
    }

    #[test]
    fn parse_test_2() {
        let tokens = lexer::tokenize("দেখাও -৫৩.৬ + ৬;".chars().collect());
        let ast = parse(tokens);
        let expected_ast = Stmt::Print(Expr::Addition(Binary {
            operator: TokenKind::Plus,
            left: Box::new(Expr::Primary(Primary::Num(-53.6))),
            right: Box::new(Expr::Primary(Primary::Num(6.0))),
        }));
        assert_eq!(expected_ast, ast[0]);
    }

    #[test]
    fn parse_test_3() {
        let tokens = lexer::tokenize("দেখাও \"this is a test\";".chars().collect());
        let ast = parse(tokens);
        let expected_ast = Stmt::Print(Expr::Primary(Primary::String(String::from("this is a test"))));
        assert_eq!(expected_ast, ast[0]);
    }

    #[test]
    fn parse_test_4() {
        let tokens = lexer::tokenize("দেখাও ১ + ৩ * ২;".chars().collect());
        let ast = parse(tokens);
        let expected_ast = Stmt::Print(Expr::Addition(Binary {
            operator: TokenKind::Plus,
            left: Box::from(Expr::Primary(Primary::Num(1.0))),
            right: Box::from(Expr::Multiplication(Binary {
                operator: TokenKind::Multiply,
                left: Box::from(Expr::Primary(Primary::Num(3.0))),
                right: Box::from(Expr::Primary(Primary::Num(2.0))),
            }))
        }));
        assert_eq!(expected_ast, ast[0]);
    }

    #[test]
    fn parse_test_5() {
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
    fn parse_test_6() {
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
    fn parse_test_7() {
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
    fn parse_test_8() {
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
    fn parse_test_9() {
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
    fn parse_test_10() {
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
    fn parse_test_11() {
        let tokens = lexer::tokenize("দেখাও সত্য & মিথ্যা;".chars().collect());
        let ast = parse(tokens);
        let expected_ast = Stmt::Print(Expr::And(And {
            left: Box::from(Expr::Primary(Primary::Bool(true))),
            right: Box::from(Expr::Primary(Primary::Bool(false))),
        }));

        assert_eq!(expected_ast, ast[0]);
    }

    #[test]
    fn parse_test_12() {
        let tokens = lexer::tokenize("দেখাও সত্য | মিথ্যা;".chars().collect());
        let ast = parse(tokens);
        let expected_ast = Stmt::Print(Expr::Or(Or {
            left: Box::from(Expr::Primary(Primary::Bool(true))),
            right: Box::from(Expr::Primary(Primary::Bool(false))),
        }));

        assert_eq!(expected_ast, ast[0]);
    }

    #[test]
    fn parse_test_13() {
        let tokens = lexer::tokenize("দেখাও !সত্য;".chars().collect());
        let ast = parse(tokens);
        let expected_ast = Stmt::Print(Expr::Unary(Unary {
            operator: TokenKind::Not,
            right: Box::from(Expr::Primary(Primary::Bool(true))),
        }));

        assert_eq!(expected_ast, ast[0]);
    }
}