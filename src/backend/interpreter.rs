use std::collections::HashMap;
use crate::frontend::parser;
use crate::frontend::lexer::Token;
use crate::frontend::lexer::TokenKind;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum ExprResult {
    Num(f64),
    Bool(bool),
    String(String),
}

struct Interpreter {
    current: usize,
    statements: Vec<parser::Stmt>,
    envs: Vec<HashMap<String, Option<ExprResult>>>,
}

impl Interpreter {
    fn new(statements: Vec<parser::Stmt>) -> Interpreter {
        Interpreter {
            current: 0,
            statements,
            envs: vec![HashMap::new()],
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
            parser::Stmt::BlockStart => {
                self.current += 1;

                // creating new scope
                self.envs.push(HashMap::new());
            },
            parser::Stmt::BlockEnd => {
                self.current += 1;

                // BlockEnd means all statements in this blocks scope were interpreted
                // so destroying scope created by Stmt::BlockStart
                self.envs.pop();
            }
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

        let mut var_found_at_env_index: i32 = (self.envs.len() - 1) as i32;
        if assign_stmt.kind == parser::AssignmentKind::Reassignment {
            for env in self.envs.iter().rev() {
                if env.contains_key(&var_key) && assign_stmt.init_value.is_some() {
                    break;
                } else {
                    var_found_at_env_index -= 1;
                }
            }
            let init_expr = assign_stmt.init_value.clone().unwrap();
            let init_value = self.interpret_expr(init_expr);

            if var_found_at_env_index < 0 {
                panic!("Variable wasn't initialized {:#}", var_key);
            } else {
                self.envs[var_found_at_env_index as usize].insert(var_key.clone(), Some(init_value));
            }
        }

        if assign_stmt.kind == parser::AssignmentKind::FirstAssignment {
            match assign_stmt.init_value {
                Some(expr) => {
                    let init_value = self.interpret_expr(expr);

                    let env_i = self.envs.len() - 1;
                    let current_env = &mut self.envs[env_i];
                    current_env.insert(var_key.clone(), Some(init_value));
                },
                _ => {
                    let env_i = self.envs.len() - 1;
                    let current_env = &mut self.envs[env_i];
                    current_env.insert(var_key.clone(), None);
                },
            }
        }

        self.current += 1;
    }

    fn interpret_expr(&mut self, expr: parser::Expr) -> ExprResult {
        match expr {
            parser::Expr::Primary(p) => self.interpret_primary_expr(p),
            parser::Expr::Unary(u_expr) => self.interpret_unary_expr(u_expr),
            parser::Expr::And(and_expr) => self.interpret_and_expr(and_expr),
            parser::Expr::Or(or_expr) => self.interpret_or_expr(or_expr),
            parser::Expr::Equality(eq_expr) => self.interpret_eq_expr(eq_expr),
            parser::Expr::Comparison(comp_expr) => self.interpret_comp_expr(comp_expr),
            parser::Expr::AddOrSub(addsub_expr) => self.interpret_addsub_expr(addsub_expr),
            parser::Expr::MulOrDiv(muldiv_expr) => self.interpret_muldiv_expr(muldiv_expr),
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
            //_ => panic!("Primary interpretation not implemented\n Debug Primary: {:#?}", p),
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

    fn interpret_and_expr(&mut self, and_expr: parser::And) -> ExprResult {
        let right_expr_val = self.interpret_expr(*and_expr.right);
        let left_expr_val = self.interpret_expr(*and_expr.left);

        if let ExprResult::Bool(right)  = right_expr_val {
           if let ExprResult::Bool(left) = left_expr_val {
               return ExprResult::Bool(right && left);
           }
        }

        panic!("Unsupported operation on type");
    }

    fn interpret_or_expr(&mut self, or_expr: parser::Or) -> ExprResult {
        let right_expr_val = self.interpret_expr(*or_expr.right);
        let left_expr_val = self.interpret_expr(*or_expr.left);

        if let ExprResult::Bool(right)  = right_expr_val {
            if let ExprResult::Bool(left) = left_expr_val {
                return ExprResult::Bool(right || left);
            }
        }

        panic!("Unsupported operation on type");
    }

    fn interpret_addsub_expr(&mut self, addsub_expr: parser::Binary) -> ExprResult {
        let right_expr_val = self.interpret_expr(*addsub_expr.right);
        let left_expr_val = self.interpret_expr(*addsub_expr.left);

        if let ExprResult::Num(right)  = right_expr_val {
            if let ExprResult::Num(left) = left_expr_val {
                match addsub_expr.operator {
                    TokenKind::Plus => return ExprResult::Num(left + right),
                    TokenKind::Minus => return ExprResult::Num(left - right),
                    _ => panic!(),
                }
            }
        }

        panic!("Unsupported operation on type");
    }

    fn interpret_muldiv_expr(&mut self, muldiv_expr: parser::Binary) -> ExprResult {
        let right_expr_val = self.interpret_expr(*muldiv_expr.right);
        let left_expr_val = self.interpret_expr(*muldiv_expr.left);

        if let ExprResult::Num(right)  = right_expr_val {
            if let ExprResult::Num(left) = left_expr_val {
                match muldiv_expr.operator {
                    TokenKind::Multiply => return ExprResult::Num(left * right),
                    TokenKind::Division => return ExprResult::Num(left / right),
                    _ => panic!(),
                }
            }
        }

        panic!("Unsupported operation on type");
    }

    fn interpret_eq_expr(&mut self, eq_expr: parser::Binary) -> ExprResult {
        let right_expr_val = self.interpret_expr(*eq_expr.right);
        let left_expr_val = self.interpret_expr(*eq_expr.left);

        match eq_expr.operator {
            TokenKind::EqualEqual => ExprResult::Bool(right_expr_val == left_expr_val),
            TokenKind::NotEqual =>  ExprResult::Bool(right_expr_val != left_expr_val),
            _ => panic!()
        }
    }

    fn interpret_comp_expr(&mut self, comp_expr: parser::Binary) -> ExprResult {
        let right_expr_val = self.interpret_expr(*comp_expr.right);
        let left_expr_val = self.interpret_expr(*comp_expr.left);

        if let ExprResult::Bool(_b) = right_expr_val {
            if let ExprResult::Bool(_b) = left_expr_val {
                panic!();
            }
            panic!();
        }

        match comp_expr.operator {
            TokenKind::GreaterThan => ExprResult::Bool(left_expr_val > right_expr_val),
            TokenKind::GreaterThanOrEqual =>  ExprResult::Bool(left_expr_val >= right_expr_val),
            TokenKind::LessThan =>  ExprResult::Bool(left_expr_val < right_expr_val),
            TokenKind::LessThanOrEqual =>  ExprResult::Bool(left_expr_val <= right_expr_val),
            _ => panic!()
        }
    }

    fn interpret_var(&mut self, v: Token) -> ExprResult {
        let var_key: String = v.lexeme.clone().into_iter().collect();

        for env in self.envs.iter_mut().rev() {
            let expr_result = env.get(&*var_key);
            if expr_result.is_some() {
                match expr_result.unwrap() {
                    Some(var_value) => return var_value.clone(),
                    None => panic!("Variable wasn't initialized {:#?}", v.lexeme),
                }
            }
        }
        panic!("Variable wasn't initialized {:#?}", v.lexeme);
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