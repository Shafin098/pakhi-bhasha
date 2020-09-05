use crate::frontend::parser;
use std::collections::HashMap;
use crate::frontend::lexer::Token;
use crate::frontend::lexer::TokenKind;
use crate::frontend::parser::Stmt::Expression;

enum ExprResult {
    Num(f64),
    Bool(bool),
    String(String),
}

struct Interpreter {
    current: usize,
    statements: Vec<parser::Stmt>,
    environment: HashMap<String, Option<parser::Expr>>
}

impl Interpreter {
    fn new(statements: Vec<parser::Stmt>) -> Interpreter {
        Interpreter {
            current: 0,
            statements,
            environment: HashMap::new(),
        }
    }

    fn run(&mut self) {
        while self.statements[self.current] != parser::Stmt::EOS {
            self.interpret();
        }
    }

    fn interpret(&mut self) {
        match self.statements[self.current].clone() {
            parser::Stmt::Print(expr) => self.interpret_print_stmt(expr),
            parser::Stmt::Assignment(assign_stmt) => self.interpret_assign_stmt(assign_stmt),
            _ => panic!("Interpreter error\n Debug Statement {:#?}", self.statements[self.current]),
        }
    }

    fn interpret_print_stmt(&mut self, expr: parser::Expr) {
        match self.interpret_expr(expr) {
            ExprResult::Num(n) => println!("{}", self.to_bn_num(n)),
            ExprResult::Bool(b) => println!("{}", self.to_bn_bool(b)),
            ExprResult::String(s) => println!("{}", s),
        }
        self.current += 1;
    }

    fn interpret_assign_stmt(&mut self, assign_stmt: parser::Assignment) {
        let var_key: String = assign_stmt.var_name.lexeme.clone().into_iter().collect();
        if assign_stmt.kind == parser::AssignmentKind::Reassignment {
            if self.environment.contains_key(&var_key) {
                self.environment.insert(var_key.clone(), assign_stmt.init_value.clone());
            } else {
                panic!("Variable {:#?} wasn't initialized", assign_stmt.var_name.lexeme);
            }
        }

        self.current += 1;

        self.environment.insert(var_key.clone(), assign_stmt.init_value);
    }

    fn interpret_expr(&mut self, expr: parser::Expr) -> ExprResult {
        match expr {
            parser::Expr::Primary(p) => self.interpret_primary_expr(p),
            parser::Expr::Unary(u_expr) => self.interpret_unary_expr(u_expr),
            _ => panic!("Expr interpretation not implemented\n Debug Expr: {:#?}", expr)
        }
    }

    fn interpret_primary_expr(&mut self, p: parser::Primary) -> ExprResult {
        match p {
            parser::Primary::String(s) => ExprResult::String(s),
            parser::Primary::Num(n) => ExprResult::Num(n),
            parser::Primary::Bool(b) => ExprResult::Bool(b),
            parser::Primary::Var(v) => self.interpret_var(v),
            parser::Primary::Group(expr) => self.interpret_expr(*expr),
            _ => panic!("Primary interpretation not implemented\n Debug Primary: {:#?}", p),
        }
    }

    fn interpret_unary_expr(&mut self, u_expr: parser::Unary) -> ExprResult {
        let expr_val = self.interpret_expr(*u_expr.right);
        match expr_val {
            ExprResult::Num(n) => {
                if u_expr.operator == TokenKind::Minus {
                    return ExprResult::Num(n * -1.0);
                }
                panic!("Unsupported operation on type");
            },
            ExprResult::Bool(b) => {
                if u_expr.operator == TokenKind::Not {
                    return ExprResult::Bool(!b);
                }
                panic!("Unsupported operation on type");
            },
            _ => panic!("Unsupported operation on type")
        }
    }

    fn interpret_var(&mut self, v: Token) -> ExprResult {
        let var_key: String = v.lexeme.clone().into_iter().collect();
        let var_expression = self.environment.get(&var_key).unwrap();
        match var_expression.clone() {
            Some(expr) => {
                return self.interpret_expr(expr)
            },
            None => panic!("Variable wasn't initialized {:#?}", v.lexeme),
        }
    }

    fn to_bn_num(&self, n: f64) -> String {
        let n_chars: Vec<char> = n.to_string().chars().collect();

        let mut bangla_num_string = String::new();
        for digit in n_chars {
           match digit {
               '-' => bangla_num_string.push('-'),
               '.' => bangla_num_string.push('.'),
               '0' => bangla_num_string.push('০'),
               '1' => bangla_num_string.push('১'),
               '2' => bangla_num_string.push('২'),
               '3' => bangla_num_string.push('৩'),
               '4' => bangla_num_string.push('৪'),
               '5' => bangla_num_string.push('৫'),
               '6' => bangla_num_string.push('৬'),
               '7' => bangla_num_string.push('৭'),
               '8' => bangla_num_string.push('৮'),
               '9' => bangla_num_string.push('৯'),
               _ => panic!("Debug {}", digit),
           }
        }

        bangla_num_string
    }
    fn to_bn_bool(&self, b: bool) -> String {
        match b {
            true => "সত্য".to_string(),
            false => "মিথ্যা".to_string(),
        }
    }
}

pub fn run(ast: Vec<parser::Stmt>) {
    let mut interpreter = Interpreter::new(ast);
    interpreter.run();
}