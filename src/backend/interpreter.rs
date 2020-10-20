use std::collections::HashMap;
use crate::frontend::parser;
use crate::frontend::lexer::Token;
use crate::frontend::lexer::TokenKind;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum DataType {
    Num(f64),
    Bool(bool),
    String(String),
    // Array variant of DataType enum only stores the index of the actual array from arrays
    // field in Interpreter, so multiple array reference implementation is easy.
    Array(usize),
    Function(Func),
    Nil,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Func {
    starting_statement: usize,
    args: Vec<String>,
}

#[derive(Debug)]
struct LoopEnv {
    start: usize,
    // this is needed to destroy envs created inside loop when using continue or break
    total_envs_at_loop_creation: usize,
}

struct Interpreter {
    current: usize,
    statements: Vec<parser::Stmt>,
    loops: Vec<LoopEnv>,
    return_addrs: Vec<usize>,
    envs: Vec<HashMap<String, Option<DataType>>>,
    previous_if_was_executed: Vec<bool>,
    arrays: Vec<Vec<DataType>>,
}

impl Interpreter {
    fn new(statements: Vec<parser::Stmt>) -> Interpreter {
        Interpreter {
            current: 0,
            statements,
            loops: Vec::new(),
            return_addrs: Vec::new(),
            envs: vec![HashMap::new()],
            previous_if_was_executed: Vec::new(),
            arrays: Vec::new(),
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
            parser::Stmt::PrintNoEOL(expr) => self.interpret_print_no_eol(expr),
            parser::Stmt::Assignment(assign_stmt) => self.interpret_assign_stmt(assign_stmt),
            parser::Stmt::If(cond_expr) => self.interpret_if_stmt(cond_expr),
            parser::Stmt::Else => self.interpret_else_stmt(),
            parser::Stmt::FuncDef => self.interpret_funcdef(),
            parser::Stmt::Expression(expr) => {
                self.interpret_expr(expr);
                self.current += 1;
            },
            parser::Stmt::Loop => {
                // consuming loop
                self.current += 1;

                // saving loop start to reuse in continue statement
                self.loops.push(LoopEnv { start: self.current, total_envs_at_loop_creation: self.envs.len()});

            },
            parser::Stmt::Continue => {
                // destroying envs that was created inside loop
                let last_loop_env_index = self.loops.len() - 1;
                let total_envs_created_inside_loop = self.envs.len() - self.loops[last_loop_env_index].total_envs_at_loop_creation;
                for _ in 0..total_envs_created_inside_loop {
                    self.envs.pop();
                }

                let loop_start = self.loops[last_loop_env_index].start;

                self.current = loop_start;
            },
            parser::Stmt::Break => {
                self.current += 1;

                // destroying envs that was created inside loop
                let last_loop_env_index = self.loops.len() - 1;

                let total_envs_created_inside_loop = self.envs.len() - self.loops[last_loop_env_index].total_envs_at_loop_creation;
                for _ in 0..total_envs_created_inside_loop {
                    self.envs.pop();
                }
                // destroying loop env
                self.loops.pop();

                let mut stack: Vec<char> = Vec::new();
                loop {
                    if self.statements[self.current] == parser::Stmt::Loop {
                        stack.push('{');
                    }

                    if self.statements[self.current] == parser::Stmt::Continue {
                        stack.pop();
                        if stack.is_empty() {
                            // consuming Stmt::Continue
                            self.current += 1;
                            break;
                        }
                    }

                    // skipping statements in block of loop
                    self.current += 1;
                }
            },
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

    fn interpret_print_no_eol(&mut self, expr: parser::Expr) {
        match self.interpret_expr(expr) {
            DataType::Num(n) => print!("{}", self.to_bn_num(n)),
            DataType::Bool(b) => print!("{}", self.to_bn_bool(b)),
            DataType::String(s) => print!("\"{}\"", s),
            DataType::Array(arr_i) => {
                let mut elems: Vec<(usize, DataType)>  = Vec::new();
                for (i, elem) in self.arrays[arr_i].iter().enumerate() {
                    elems.push((i, elem.clone()));
                }
                print!("[");
                for (i, elem) in elems {
                    self.print_datatype(elem.clone());
                    if (i+1) < self.arrays[arr_i].len() {
                        print!(", ")
                    }
                }
                print!("]");
            }
            _ => panic!("Datatype isn't implemented"),
        }
        self.current += 1;
    }

    fn print_datatype(&mut self, data: DataType) {
        match data {
            DataType::Num(n) => print!("{}", self.to_bn_num(n)),
            DataType::Bool(b) => print!("{}", self.to_bn_bool(b)),
            DataType::String(s) => print!("\"{}\"", s),
            DataType::Array(arr_i) => {
                let mut elems: Vec<(usize, DataType)>  = Vec::new();
                for (i, elem) in self.arrays[arr_i].iter().enumerate() {
                    elems.push((i.clone(), elem.clone()));
                }
                print!("[");
                for (i, elem) in elems.clone() {
                    self.print_datatype(elem.clone());
                    if (i+1) < self.arrays[arr_i].len() {
                        print!(", ")
                    }
                }
                print!("]");
            }
            _ => panic!("Datatype isn't implemented"),
        }
    }

    fn interpret_print_stmt(&mut self, expr: parser::Expr) {
        match self.interpret_expr(expr) {
            DataType::Num(n) => println!("{}", self.to_bn_num(n)),
            DataType::Bool(b) => println!("{}", self.to_bn_bool(b)),
            DataType::String(s) => println!("\"{}\"", s),
            DataType::Array(arr_i) => {
                let mut elems: Vec<(usize, DataType)>  = Vec::new();
                for (i, elem) in self.arrays[arr_i].iter().enumerate() {
                    elems.push((i, elem.clone()));
                }
                print!("[");
                for (i, elem) in elems.clone() {
                    self.print_datatype(elem.clone());
                    if (i+1) < self.arrays[arr_i].len() {
                        print!(", ")
                    }
                }
                println!("]");
            }
            _ => panic!("Datatype printing isn't implemented"),
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

            if var_found_at_env_index >= 0 {
                if assign_stmt.indexes.is_empty() {
                    // only simple variable assignment
                    self.envs[var_found_at_env_index as usize].insert(var_key.clone(), Some(init_value));
                } else {
                    // assignment to element in a array

                    // effective_index is index of deepest nested array, to which init_val will be assigned
                    let effective_index = self.interpret_expr(assign_stmt.indexes.last().unwrap().clone());
                    let mut evaluated_index_exprs: Vec<usize> = Vec::new();
                    for i in 0..assign_stmt.indexes.len() {
                        let index = self.interpret_expr(assign_stmt.indexes[i].clone());
                        match  index {
                            DataType::Array(arr_i) => {
                                match self.arrays[arr_i][0] {
                                    DataType::Num(i) => evaluated_index_exprs.push(i as usize),
                                    _ => panic!(),
                                }
                            }
                            _ => panic!(),
                        }
                    }

                    let var = self.envs[var_found_at_env_index as usize]
                        .get(var_key.as_str()).unwrap();

                    match var {
                        Some(DataType::Array(i)) => {
                            let array_copy = self.arrays.clone();
                            let arr = self.arrays.get_mut(*i).unwrap();

                            if assign_stmt.indexes.len() == 1 {
                                match effective_index {
                                    DataType::Array(j) => {
                                        let a = array_copy[j].clone();
                                        match a[0] {
                                            DataType::Num(n) => arr[n as usize] = init_value,
                                            _ => panic!(),
                                        }
                                    },
                                    _ => panic!(),
                                }
                                self.current += 1;
                                return;
                            }

                            let mut assignee: DataType = arr.get(evaluated_index_exprs.get(0).unwrap().clone()).unwrap().clone();

                            for i in 1..evaluated_index_exprs.len() {
                                if i == evaluated_index_exprs.len() - 1 {
                                    match assignee {
                                        DataType::Array(arr_i) => {
                                            //let a = self.arrays.get_mut(arr_i).unwrap();
                                            let index = evaluated_index_exprs.get(i).unwrap();
                                            self.arrays[arr_i][index.clone()] = init_value.clone();
                                            //a[index.clone()] = init_value.clone();
                                            break;
                                        },
                                        _ => panic!("Cannot assign at index if data type is not array"),
                                    }
                                } else {
                                    match assignee {
                                        DataType::Array(arr_i) => {
                                            let a = self.arrays.get_mut(arr_i).unwrap();
                                            let index = evaluated_index_exprs.get(i).unwrap();
                                            assignee = a.get(index.clone()).unwrap().clone();
                                        },
                                        _ => panic!("Cannot index if not array"),
                                    }
                                }
                            }
                        },
                        _ => panic!("Variable wasn't initialized {:#}", var_key),
                    }
                }
            } else {
                panic!("Variable wasn't initialized {:#}", var_key);
            }
        }

        if assign_stmt.kind == parser::AssignmentKind::FirstAssignment {
            match assign_stmt.init_value {
                Some(expr) => {
                    let init_value = self.interpret_expr(expr);

                    let env_i = self.envs.len() - 1;
                    let current_env = &mut self.envs[env_i];
                    current_env.insert(var_key, Some(init_value));
                },
                _ => {
                    let env_i = self.envs.len() - 1;
                    let current_env = &mut self.envs[env_i];
                    current_env.insert(var_key, None);
                },
            }
        }

        self.current += 1;
    }

    fn interpret_funcdef(&mut self) {
        // consuming function definition statement
        self.current += 1;

        if let parser::Stmt::Expression(parser::Expr::Call(function)) = self.statements[self.current].clone() {
            match *function.expr {
                parser::Expr::Primary(parser::Primary::Var(func_token)) => {
                    let func_name: String = func_token.lexeme.iter().collect();
                    let func_args = function.arguments;
                    let mut func_args_name: Vec<String> = Vec::new();

                    for arg_expr in func_args {
                        match arg_expr {
                            parser::Expr::Primary(parser::Primary::Var(name_token)) => {
                                func_args_name.push(name_token.lexeme.iter().collect());
                            },
                            _ => panic!(),
                        }
                    }

                    let func = Func {
                        starting_statement: self.current + 1,
                        args: func_args_name,
                    };

                    let current_env_i = self.envs.len() - 1;
                    self.envs[current_env_i].insert(func_name.clone(), Some(DataType::Function(func)));
                },
                _ => panic!(),
            }
        } else {
            panic!();
        }

        // consuming function name and args statement (Expr::Call)
        self.current += 1;

        // skipping all statements in function body
        // statements in func body is not executed during func definition
        self.skip_block();

        // consuming return statement
        if let parser::Stmt::Return(_) = self.statements[self.current].clone() {
           self.current += 1;
        } else {
            panic!("Return expected");
        }
    }

    fn interpret_if_stmt(&mut self, expr: parser::Expr) {
        // consuming if token
        self.current += 1;

        let if_condition_expr = self.interpret_expr(expr);

        if let DataType::Bool(condition) = if_condition_expr {
            if condition == false {
                self.previous_if_was_executed.push(false);
                // condition expression of if statement is false so skipping next block statement
                self.skip_block_in_if();
            } else {
                self.previous_if_was_executed.push(true);
            }
        } else { panic!("If condition expression must evaluate to boolean value") }

    }

    fn interpret_else_stmt(&mut self) {
        assert!(!self.previous_if_was_executed.is_empty());

        // consuming else token
        self.current += 1;

        let last_if_condition_index = self.previous_if_was_executed.len() - 1;
        if self.previous_if_was_executed[last_if_condition_index] {
            self.skip_block_in_if();
        }
        self.previous_if_was_executed.pop();
    }

    fn skip_block(&mut self) {
        let mut stack: Vec<char> = Vec::new();

        loop {
            if self.statements[self.current] == parser::Stmt::BlockStart {
                stack.push('{');
            }

            if self.statements[self.current] == parser::Stmt::BlockEnd {
                let previous = stack.pop();
                match previous {
                    Some(_) => {
                        if stack.is_empty() {
                            // consuming Stmt::BlockEnd
                            self.current += 1;
                            break;
                        }
                    },
                    None => panic!("Syntax error"),
                }
            }

            // skipping statements in block
            self.current += 1;
        }
    }

    fn skip_block_in_if(&mut self) {
        self.skip_block();

        if self.statements[self.current] != parser::Stmt::Else {
            self.previous_if_was_executed.pop();
        }
    }

    fn interpret_expr(&mut self, expr: parser::Expr) -> DataType {
        match expr {
            parser::Expr::Primary(p) => self.interpret_primary_expr(p),
            parser::Expr::Unary(u_expr) => self.interpret_unary_expr(u_expr),
            parser::Expr::And(and_expr) => self.interpret_and_expr(and_expr),
            parser::Expr::Or(or_expr) => self.interpret_or_expr(or_expr),
            parser::Expr::Equality(eq_expr) => self.interpret_eq_expr(eq_expr),
            parser::Expr::Comparison(comp_expr) => self.interpret_comp_expr(comp_expr),
            parser::Expr::AddOrSub(addsub_expr) => self.interpret_addsub_expr(addsub_expr),
            parser::Expr::MulOrDivOrRemainder(muldiv_expr) => self.interpret_muldiv_remainder_expr(muldiv_expr),
            parser::Expr::Call(function) => self.interpret_func_call_expr(function),
            parser::Expr::ArrayIndexing(identifier, i) => self.interpret_arr_indexing(identifier, i),
        }
    }

    fn interpret_arr_indexing(&mut self, indentifier: Box<parser::Expr>, index: Box<parser::Expr>) -> DataType {
        let identifier = self.interpret_expr(*indentifier);
        let index = self.interpret_expr(*index);

        match (identifier, index) {
            (DataType::Array(arr_i), DataType::Num(i)) => {
                let arr = self.arrays[arr_i].clone();
                return arr[i as usize].clone()
            },
            (_, DataType::Num(_)) => panic!("Indexing only possible with array"),
            (DataType::Array(_), _) => panic!("Array index must evaluate to number type"),
            _ => panic!("Invalid indexing format"),
        }
    }

    fn built_in_fn_list_push(&mut self, f: &parser::FunctionCall) -> DataType {
        if f.arguments.len() == 2 {
            let list = self.interpret_expr(f.arguments[0].clone());

            if let DataType::Array(index) = list {
                let push_value = self.interpret_expr(f.arguments[1].clone());
                let actual_list = self.arrays.get_mut(index).unwrap();
                actual_list.push(push_value);
            } else {
                panic!("Datatype must be array to push value")
            }

        } else if f.arguments.len() == 3 {
            let list = self.interpret_expr(f.arguments[0].clone());

            if let DataType::Array(index) = list {
                let push_at = self.interpret_expr(f.arguments[1].clone());
                let push_value = self.interpret_expr(f.arguments[2].clone());
                let actual_list = self.arrays.get_mut(index).unwrap();

                if let DataType::Num(push_at_i_f) = push_at {
                    let push_at_u = push_at_i_f as usize;
                    actual_list.insert(push_at_u, push_value);
                } else { panic!("Index must evaluate to number type"); }

            } else { panic!("Datatype must be array to push value") }

        } else { panic!("Function requires two arguments") }

        return DataType::Nil;
    }

    fn built_in_fn_list_pop(&mut self, f: &parser::FunctionCall) -> DataType {
        if f.arguments.len() == 1 {
            let list = self.interpret_expr(f.arguments[0].clone());

            if let DataType::Array(index) = list {
                let actual_list = self.arrays.get_mut(index).unwrap();
                actual_list.pop();
            } else { panic!("Datatype must be array to push value")}

        } else if f.arguments.len() == 2 {
            let list = self.interpret_expr(f.arguments[0].clone());

            if let DataType::Array(index) = list {
                let pop_at = self.interpret_expr(f.arguments[1].clone());
                let actual_list = self.arrays.get_mut(index).unwrap();

                if let DataType::Num(pop_at_i_f) = pop_at {
                    let pop_at_i = pop_at_i_f as usize;
                    actual_list.remove(pop_at_i);
                }

            } else { panic!("Datatype must be array to push value") }

        } else { panic!("Function requires one argument") }

        return DataType::Nil;
    }

    fn built_in_fn_read_line(&mut self, f: &parser::FunctionCall) -> DataType {
        if f.arguments.len() == 0 {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Error reading from stdin");
            return DataType::String(input.trim_end().into());
        } else { panic!("Function requires zero argument") }

    }

    fn interpret_func_call_expr(&mut self, f: parser::FunctionCall) -> DataType {
        let env_count_before_fn_call = self.envs.len();

        match *f.expr.clone() {
            parser::Expr::Primary(parser::Primary::Var(func_token)) => {
                // if any of the if condition equals true its a built in function
                // user defined  functions ar handled in else clause
                let func_name = func_token.lexeme.clone();

                if func_name == "_লিস্ট-পুশ".chars().collect::<Vec<char>>() {
                    return self.built_in_fn_list_push(&f);
                } else if func_name == "_লিস্ট-পপ".chars().collect::<Vec<char>>(){
                    return self.built_in_fn_list_pop(&f);
                } else if func_name == "_রিড-লাইন".chars().collect::<Vec<char>>() {
                    return self.built_in_fn_read_line(&f);
                } else {
                    // assumes function was user-defined function
                    // this block checks if function was declared,
                    // sets up environment, inserts args to new environment
                    // and saves return address for function call
                    let func = self.interpret_var(func_token);

                    if let DataType::Function(func) = func {
                        let mut root_env: HashMap<String, Option<DataType>> = HashMap::new();
                        for i in 0..func.args.len() {
                            if i < f.arguments.len() {
                                root_env.insert(func.args[i].clone(), Option::from(self.interpret_expr(f.arguments[i].clone())));
                            } else {
                                // not enough arguments passed so assigning Nil
                                root_env.insert(func.args[i].clone(), Option::from(DataType::Nil));
                            }
                        }

                        // root key indicates its a functions root env
                        root_env.insert("root".to_string(), Some(DataType::Nil));

                        // creating root_envs
                        self.envs.push(root_env);

                        self.return_addrs.push(self.current);

                        // pointing current to functions starting statement
                        self.current = func.starting_statement;
                    } else {
                        panic!("Function not Declared");
                    }
                }

            },
            _ => panic!(),
        }

        // jumping to function start and starting executing statements in function body

        assert_eq!(parser::Stmt::BlockStart, self.statements[self.current]);
        // interpreting all statements inside function body
        // assuming self.current was set at function start
        loop {
            if let parser::Stmt::Return(_) = self.statements[self.current].clone() {
                break;
            } else {
                self.interpret();
            }
        }

        if let parser::Stmt::Return(expr) = self.statements[self.current].clone() {
            let return_val = self.interpret_expr(expr);
            self.current = self.return_addrs.pop().unwrap();

            let env_count_after_fn_call = self.envs.len();
            let envs_created_inside_fn = env_count_after_fn_call - env_count_before_fn_call;
            for _ in 0..envs_created_inside_fn {
                // return can also happen mid function without reaching blockEnd '}' statement
                // so half used env must be destroyed manually
                self.envs.pop();
            }

            return return_val;
        }
        panic!();
    }

    fn interpret_primary_expr(&mut self, p: parser::Primary) -> DataType {
        match p {
            parser::Primary::Nil => DataType::Nil,
            parser::Primary::String(s) => DataType::String(s),
            parser::Primary::Num(n) => DataType::Num(n),
            parser::Primary::Bool(b) => DataType::Bool(b),
            parser::Primary::Var(v) => self.interpret_var(v),
            parser::Primary::Array(array) => {
                // this is a internal pakhi array data type representation
                let mut pakhi_array: Vec<DataType> = Vec::new();
                for elem in array {
                    pakhi_array.push(self.interpret_expr(elem));
                }

                self.arrays.push(pakhi_array);
                return DataType::Array(self.arrays.len() - 1);
            },
            parser::Primary::Group(expr) => self.interpret_expr(*expr),
        }
    }

    fn interpret_unary_expr(&mut self, u_expr: parser::Unary) -> DataType {
        let expr_val = self.interpret_expr(*u_expr.right);
        match expr_val {
            DataType::Num(n) => {
                if u_expr.operator == TokenKind::Minus {
                    return DataType::Num(n * -1.0);
                }
                panic!("Unsupported operation on type");
            },
            DataType::Bool(b) => {
                if u_expr.operator == TokenKind::Not {
                    return DataType::Bool(!b);
                }
                panic!("Unsupported operation on type");
            },
            _ => panic!("Unsupported operation on type")
        }
    }

    fn interpret_and_expr(&mut self, and_expr: parser::And) -> DataType {
        let right_expr_val = self.interpret_expr(*and_expr.right);
        let left_expr_val = self.interpret_expr(*and_expr.left);

        if let DataType::Bool(right)  = right_expr_val {
           if let DataType::Bool(left) = left_expr_val {
               return DataType::Bool(right && left);
           }
        }

        panic!("Unsupported operation on type");
    }

    fn interpret_or_expr(&mut self, or_expr: parser::Or) -> DataType {
        let right_expr_val = self.interpret_expr(*or_expr.right);
        let left_expr_val = self.interpret_expr(*or_expr.left);

        if let DataType::Bool(right)  = right_expr_val {
            if let DataType::Bool(left) = left_expr_val {
                return DataType::Bool(right || left);
            }
        }

        panic!("Unsupported operation on type");
    }

    fn interpret_addsub_expr(&mut self, addsub_expr: parser::Binary) -> DataType {
        let left_expr_val = self.interpret_expr(*addsub_expr.left);
        let right_expr_val = self.interpret_expr(*addsub_expr.right);

        match (left_expr_val, right_expr_val) {
            (DataType::Num(left), DataType::Num(right)) => {
                match addsub_expr.operator {
                    TokenKind::Plus => return DataType::Num(left + right),
                    TokenKind::Minus => return DataType::Num(left - right),
                    _ => panic!("Invalid operation"),
                }
            },
            (DataType::String(left_str), DataType::String(right_str)) => {
                if addsub_expr.operator == TokenKind::Plus {
                    return DataType::String(format!("{}{}", left_str, right_str));
                }
                panic!("Invalid operation on String");
            },
            (DataType::Array(ref mut left_arr_i), DataType::Array(ref mut right_arr_i)) => {
                let left_arr = self.arrays.get(*left_arr_i).unwrap().clone();
                let right_arr = self.arrays.get(*right_arr_i).unwrap().clone();
                if addsub_expr.operator == TokenKind::Plus {
                    let mut concatted_arr: Vec<DataType> = Vec::new();
                    for elem in left_arr {
                        concatted_arr.push(elem);
                    }
                    for elem in right_arr {
                        concatted_arr.push(elem);
                    }
                    self.arrays.push(concatted_arr);
                    return DataType::Array(self.arrays.len() - 1);
                }
                panic!("Invalid operation on Arry")
            }
            _ => panic!("Invalid operation, operand doesn't support this operation"),
        }
    }

    fn interpret_muldiv_remainder_expr(&mut self, muldiv_expr: parser::Binary) -> DataType {
        let right_expr_val = self.interpret_expr(*muldiv_expr.right);
        let left_expr_val = self.interpret_expr(*muldiv_expr.left);

        if let DataType::Num(right)  = right_expr_val {
            if let DataType::Num(left) = left_expr_val {
                match muldiv_expr.operator {
                    TokenKind::Multiply => return DataType::Num(left * right),
                    TokenKind::Division => return DataType::Num(left / right),
                    TokenKind::Remainder => return DataType::Num(left % right),
                    _ => panic!(),
                }
            }
        }

        panic!("Unsupported operation on type");
    }

    fn interpret_eq_expr(&mut self, eq_expr: parser::Binary) -> DataType {
        let evaluated_left_expr = self.interpret_expr(*eq_expr.left);
        let evaluated_right_expr = self.interpret_expr(*eq_expr.right);

        match eq_expr.operator {
            TokenKind::EqualEqual => DataType::Bool(evaluated_left_expr == evaluated_right_expr),
            TokenKind::NotEqual =>  DataType::Bool(evaluated_left_expr != evaluated_right_expr ),
            _ => panic!()
        }
    }

    fn interpret_comp_expr(&mut self, comp_expr: parser::Binary) -> DataType {
        let evaluated_left_expr = self.interpret_expr(*comp_expr.left);
        let evaluated_right_expr = self.interpret_expr(*comp_expr.right);

        match (evaluated_left_expr.clone(), evaluated_right_expr.clone()) {
            (DataType::Num(_), DataType::Num(_)) => {
                match comp_expr.operator {
                    TokenKind::GreaterThan => DataType::Bool(evaluated_left_expr > evaluated_right_expr),
                    TokenKind::GreaterThanOrEqual => DataType::Bool(evaluated_left_expr >= evaluated_right_expr),
                    TokenKind::LessThan => DataType::Bool(evaluated_left_expr < evaluated_right_expr),
                    TokenKind::LessThanOrEqual => DataType::Bool(evaluated_left_expr <= evaluated_right_expr),
                    _ => panic!()
                }
            },
            _ => panic!()
        }
    }

    fn interpret_var(&mut self, v: Token) -> DataType {
        let var_key: String = v.lexeme.clone().into_iter().collect();

        for env in self.envs.iter_mut().rev() {

            let expr_result = env.get(&*var_key);
            if expr_result.is_some() {
                match expr_result.unwrap() {
                    Some(var_value) => return var_value.clone(),
                    None => {
                        panic!("Variable wasn't initialized {:#?}", v.lexeme)
                    },
                }
            }

            // if contains root means at current env is root of the function
            if env.contains_key("root") {
                break;
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