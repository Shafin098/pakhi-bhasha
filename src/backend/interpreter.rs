use std::collections::HashMap;
use crate::common::io::{IO, RealIO};
use crate::frontend::parser;
use crate::frontend::lexer::{TokenKind, Token};
use crate::backend::built_ins::BuiltInFunctionList;

enum Index {
    List(usize),
    NamelessRecord(String),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum DataType {
    Num(f64),
    Bool(bool),
    String(String),
    // Array variant of DataType enum only stores the index of the actual array from arrays
    // field in Interpreter, so multiple array reference implementation is easy.
    List(usize),
    NamelessRecord(usize),
    Function(Func),
    Nil,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Func {
    starting_statement: usize,
    args: Vec<String>,
}

#[derive(Debug)]
struct LoopEnv {
    start: usize,
    // this is needed to destroy envs created inside loop when using continue or break
    total_envs_at_loop_creation: usize,
}

pub struct Interpreter<'a, T: IO> {
    current: usize,
    statements: Vec<parser::Stmt>,
    loops: Vec<LoopEnv>,
    return_addrs: Vec<usize>,
    envs: Vec<HashMap<String, Option<DataType>>>,
    previous_if_was_executed: Vec<bool>,
    lists: Vec<Vec<DataType>>,
    nameless_records: Vec<HashMap<String, DataType>>,
    io: &'a mut T,
    // Storing all built-in function names because when modules identifiers are renamed
    // we don't want to rename built-in functions
    built_in_functions: BuiltInFunctionList,
}

impl<T: IO> Interpreter<'_, T> {
    pub fn new(statements: Vec<parser::Stmt>, io: &mut T) -> Interpreter<T> {
        Interpreter {
            current: 0,
            statements,
            loops: Vec::new(),
            return_addrs: Vec::new(),
            envs: vec![HashMap::new()],
            previous_if_was_executed: Vec::new(),
            lists: Vec::new(),
            nameless_records: Vec::new(),
            io,
            built_in_functions: BuiltInFunctionList::new(),
        }
    }

    pub fn run(&mut self) {
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
            DataType::Num(n) => self.io.print( self.to_bn_num(n).as_str()),
            DataType::Bool(b) => self.io.print( self.to_bn_bool(b).as_str()),
            DataType::String(s) => self.io.print(s.as_str()),
            DataType::List(arr_i) => {
                let mut elems: Vec<(usize, DataType)>  = Vec::new();
                for (i, elem) in self.lists[arr_i].iter().enumerate() {
                    elems.push((i, elem.clone()));
                }
                self.io.print("[");
                for (i, elem) in elems {
                    self.print_datatype(elem.clone());
                    if (i+1) < self.lists[arr_i].len() {
                        self.io.print(", ")
                    }
                }
                self.io.print("]");
            },
            DataType::NamelessRecord(record_i) => {
                let nameless_record = self.nameless_records.get(record_i).unwrap().clone();
                self.io.print("@{");
                for (k, v) in nameless_record {
                    self.io.print(&*format!("\"{}\":", k));
                    self.print_datatype(v.clone());
                    self.io.print(",")
                }
                self.io.print("}");
            },
            _ => panic!("Datatype isn't implemented"),
        }
        self.current += 1;
    }

    fn print_datatype(&mut self, data: DataType) {
        match data {
            DataType::Num(n) => self.io.print( self.to_bn_num(n).as_str()),
            DataType::Bool(b) => self.io.print( self.to_bn_bool(b).as_str()),
            DataType::String(s) => self.io.print(s.as_str()),
            DataType::List(arr_i) => {
                let mut elems: Vec<(usize, DataType)>  = Vec::new();
                for (i, elem) in self.lists[arr_i].iter().enumerate() {
                    elems.push((i.clone(), elem.clone()));
                }
                self.io.print("[");
                for (i, elem) in elems.clone() {
                    self.print_datatype(elem.clone());
                    if (i+1) < self.lists[arr_i].len() {
                        self.io.print(", ")
                    }
                }
                self.io.print("]");
            },
            DataType::NamelessRecord(record_i) => {
                let nameless_record = self.nameless_records.get(record_i).unwrap().clone();
                self.io.print("@{");
                for (k, v) in nameless_record {
                    self.io.print(&*format!("\"{}\":", k));
                    self.print_datatype(v.clone());
                    self.io.print(",")
                }
                self.io.print("}");
            },
            _ => panic!("Datatype isn't implemented"),
        }
    }

    fn interpret_print_stmt(&mut self, expr: parser::Expr) {
        match self.interpret_expr(expr) {
            DataType::Num(n) => self.io.println(self.to_bn_num(n).as_str()),
            DataType::Bool(b) => self.io.println(self.to_bn_bool(b).as_str()),
            DataType::String(s) => self.io.println( s.as_str()),
            DataType::List(arr_i) => {
                let mut elems: Vec<(usize, DataType)>  = Vec::new();
                for (i, elem) in self.lists[arr_i].iter().enumerate() {
                    elems.push((i, elem.clone()));
                }
                self.io.print("[");
                for (i, elem) in elems.clone() {
                    self.print_datatype(elem.clone());
                    if (i+1) < self.lists[arr_i].len() {
                        self.io.print(", ")
                    }
                }
                self.io.println("]");
            },
            DataType::NamelessRecord(record_i) => {
                let nameless_record = self.nameless_records.get(record_i).unwrap().clone();
                self.io.print("@{");
                for (k, v) in nameless_record {
                    self.io.print(&*format!("\"{}\":", k));
                    self.print_datatype(v.clone());
                    self.io.print(",")
                }
                self.io.println("}");
            },
            _ => panic!("Datatype printing isn't implemented"),
        }
        self.current += 1;
    }

    fn interpret_assign_stmt(&mut self, assign_stmt: parser::Assignment) {
        let var_key: String = assign_stmt.var_name.lexeme.clone().into_iter().collect();

        match assign_stmt.kind {
            parser::AssignmentKind::FirstAssignment => self.create_new_var(var_key, assign_stmt),
            parser::AssignmentKind::Reassignment => self.reassign_to_old_var(var_key, assign_stmt),
        }

        self.current += 1;
    }

    fn create_new_var(&mut self, var_key: String, assign_stmt: parser::Assignment) {
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
                current_env.insert(var_key, Some(DataType::Nil));
            },
        }
    }

    fn reassign_to_old_var(&mut self, var_key: String, assign_stmt: parser::Assignment) {
        let init_expr = assign_stmt.init_value.clone().unwrap();
        let init_value = self.interpret_expr(init_expr);

        // if variable wasn't found it evaluates to any negative number
        let var_found_at_env_index: i32 = self.find_var_env_index(var_key.clone(), assign_stmt.init_value.clone());

        if var_found_at_env_index >= 0 {
            if assign_stmt.indexes.is_empty() {
                // only simple variable assignment
                self.envs[var_found_at_env_index as usize].insert(var_key, Some(init_value));
            } else {
                // assignment to element in a list or record
                self.reassign_to_list_or_record(assign_stmt, var_key, var_found_at_env_index, init_value);
            }
        } else {
            panic!("Variable wasn't declared {:#}", var_key);
        }
    }

    // reassign a value to a list or record at provided index
    fn reassign_to_list_or_record(&mut self, assign_stmt: parser::Assignment, var_key: String, var_found_at_env_index: i32, init_value: DataType) {
        // effective_index is index of deepest nested array, to which init_val will be assigned
        let effective_index = self.interpret_expr(assign_stmt.indexes.last().unwrap().clone());
        let evaluated_indexes: Vec<Index> = self.evaluate_all_indexes(assign_stmt.indexes.clone());

        let var = self.get_var_from_env(var_key.as_str(), var_found_at_env_index as usize);

        match var {
            Some(DataType::List(i)) => {
                if assign_stmt.indexes.len() == 1 {
                    // single dimensional list
                    // changing list element at only one level deep
                    self.list_single_dim_assign(i, effective_index, init_value);
                } else {
                    // multidimensional array so need to traverse nested list ore record
                    self.list_multi_dim_assign(i, evaluated_indexes, init_value.clone());
                }
            },
            Some(DataType::NamelessRecord(record_ref)) => {
                if assign_stmt.indexes.len() == 1 {
                    // single dimensional list
                    // changing list element at only one level deep
                    self.record_single_dim_assign(record_ref, effective_index, init_value);
                } else {
                    // multidimensional array so need to traverse nested list ore record
                    self.record_multi_dim_assign(record_ref, evaluated_indexes, init_value.clone());
                }
            },
            _ => panic!("Variable wasn't declared {:#}", var_key),
        }
    }

    fn list_single_dim_assign(&mut self, list_ref: usize, index: DataType, init_value: DataType) {
        match index {
            DataType::List(j) => {
                let a = self.lists[j].clone();
                match a[0].clone() {
                    DataType::Num(n) => {
                        let list = self.lists.get_mut(list_ref).unwrap();
                        list[n as usize] = init_value
                    },
                    _ => panic!(),
                }
            },
            _ => panic!(),
        }
    }

    fn record_single_dim_assign(&mut self, record_ref: usize, index: DataType, init_value: DataType) {
        match index {
            DataType::List(j) => {
                let a = self.lists[j].clone();
                match a[0].clone() {
                    DataType::String(key) => {
                        let record = self.nameless_records.get_mut(record_ref).unwrap();
                        record.insert(key, init_value);
                    },
                    _ => panic!("Records must be indexed by a string key"),
                }
            },
            _ => panic!(),
        }
    }

    fn list_multi_dim_assign(&mut self, list_reference: usize, evaluated_indexes: Vec<Index>, init_value: DataType) {
        let list = self.lists.get_mut(list_reference).unwrap();

        match evaluated_indexes.get(0).unwrap() {
            Index::List(list_ref) => {
                let mut assignee: DataType = list.get(list_ref.clone()).unwrap().clone();

                for i in 1..evaluated_indexes.len() {
                    if i == evaluated_indexes.len() - 1 {
                        match assignee {
                            DataType::List(arr_i) => {
                                //let a = self.arrays.get_mut(arr_i).unwrap();
                                let index = evaluated_indexes.get(i).unwrap();
                                match index {
                                    Index::List(i) => {
                                        self.lists[arr_i][i.clone()] = init_value.clone();
                                        //a[index.clone()] = init_value.clone();
                                        break;
                                    },
                                    _ => panic!()
                                }
                            }
                            _ => panic!("Cannot assign at index if data type is not array"),
                        }
                    } else {
                        match assignee {
                            DataType::List(arr_i) => {
                                let a = self.lists.get_mut(arr_i).unwrap();
                                let index = evaluated_indexes.get(i).unwrap();
                                match index {
                                    Index::List(i) => {
                                        assignee = a.get(i.clone()).unwrap().clone();
                                    },
                                    _ => panic!(),
                                }
                            },
                            _ => panic!("Cannot index if not array"),
                        }
                    }
                }
            },
            _ => panic!(),
        }
    }

    fn record_multi_dim_assign(&mut self, record_reference: usize, evaluated_indexes: Vec<Index>, init_value: DataType) {
        let record = self.nameless_records.get_mut(record_reference).unwrap();

        match evaluated_indexes.get(0).unwrap() {
            Index::NamelessRecord(key) => {
                let mut assignee: DataType = record.get(key).unwrap().clone();

                for i in 1..evaluated_indexes.len() {
                    if i == evaluated_indexes.len() - 1 {
                        match assignee {
                            DataType::NamelessRecord(record_i) => {
                                let index = evaluated_indexes.get(i).unwrap();
                                match index {
                                    Index::NamelessRecord(k) => {
                                        self.nameless_records[record_i].insert(k.clone(), init_value);
                                        break;
                                    }
                                    _ => panic!()
                                }
                            }
                            _ => panic!("Cannot assign at index if data type is not array"),
                        }
                    } else {
                        match assignee {
                            DataType::NamelessRecord(record_i) => {
                                let r = self.nameless_records.get_mut(record_i).unwrap();
                                let index = evaluated_indexes.get(i).unwrap();
                                match index {
                                    Index::NamelessRecord(k) => {
                                        assignee = r.get(k).unwrap().clone();
                                    },
                                    _ => panic!(),
                                }
                            }
                            _ => panic!("Cannot index if not array"),
                        }
                    }
                }
            }
            _ => panic!(),
        }
    }

    fn find_var_env_index(&mut self, var_key: String, init_value: Option<parser::Expr>) -> i32 {
        // if var was found at env returns its scope index
        // if not found return any integer
        let mut var_found_at_env_index: i32 = (self.envs.len() - 1) as i32;

        for env in self.envs.iter().rev() {
            if env.contains_key(&var_key) && init_value.is_some() {
                break;
            } else {
                var_found_at_env_index -= 1;
            }
        }

        return var_found_at_env_index;
    }

    fn get_var_from_env(&mut self, var_name: &str, env_index: usize) -> Option<DataType> {
        return self.envs[env_index].get(var_name).unwrap().clone();
    }

    fn evaluate_all_indexes(&mut self, index_exprs: Vec<parser::Expr>) -> Vec<Index> {
        let mut evaluated_index_exprs: Vec<Index> = Vec::new();

        for i in 0..index_exprs.len() {
            let index = self.interpret_expr(index_exprs[i].clone());
            match  index {
                DataType::List(arr_i) => {
                    match self.lists[arr_i][0].clone() {
                        DataType::Num(i) => evaluated_index_exprs.push(Index::List(i as usize)),
                        DataType::String(key) => evaluated_index_exprs.push(Index::NamelessRecord(key)),
                        _ => panic!(),
                    }
                }
                _ => panic!(),
            }
        }

        return  evaluated_index_exprs;
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
            parser::Expr::Indexing(identifier, i) => self.interpret_indexing(identifier, i),
        }
    }

    fn interpret_indexing(&mut self, indentifier: Box<parser::Expr>, index: Box<parser::Expr>) -> DataType {
        let identifier = self.interpret_expr(*indentifier);
        let index = self.interpret_expr(*index);

        match (identifier, index) {
            (DataType::List(arr_i), DataType::Num(i)) => {
                let arr = self.lists[arr_i].clone();
                return arr[i as usize].clone()
            },
            (DataType::NamelessRecord(record_i), DataType::String(key)) => {
                let nameless_record = self.nameless_records[record_i].clone();
                return nameless_record.get(&*key).unwrap().clone();
            },
            (_, DataType::Num(_)) => panic!("List only supports indexing with number"),
            (DataType::List(_), _) => panic!("List index must evaluate to number type"),
            (_, DataType::String(_)) => panic!("Record ony supports indexing with string"),
            (DataType::NamelessRecord(_), _) => panic!("Record index must evaluate to string type"),
            _ => panic!("Invalid indexing format"),
        }
    }

    fn call_built_in_function(&mut self, f: &parser::FunctionCall, func_token: &Token) -> DataType {
        let mut evaluated_arguments: Vec<DataType> = Vec::new();
        // Evaluating all arguments
        for arg in f.arguments.iter() {
            let e_a = self.interpret_expr(arg.clone());
            evaluated_arguments.push(e_a);
        }
        // Finding out which built-in function and executing that accordingly
        match self.built_in_functions.get_name(&func_token.lexeme).as_str() {
            "_স্ট্রিং" => return BuiltInFunctionList::_to_string(evaluated_arguments),
            "_সংখ্যা" => return BuiltInFunctionList::_to_num(evaluated_arguments),
            "_লিস্ট-পুশ" => return BuiltInFunctionList::_list_push(evaluated_arguments, &mut self.lists),
            "_লিস্ট-পপ" => return BuiltInFunctionList::_list_pop(evaluated_arguments, &mut self.lists),
            "_লিস্ট-লেন" => return BuiltInFunctionList::_list_len(evaluated_arguments, &mut self.lists),
            "_রিড-লাইন" => return BuiltInFunctionList::_read_line(evaluated_arguments),
            "_এরর" => {
                let error = BuiltInFunctionList::_error(evaluated_arguments);
                panic!("{}", error);
            },
            "_স্ট্রিং-স্প্লিট" => return BuiltInFunctionList::_string_split(evaluated_arguments, &mut self.lists),
            "_স্ট্রিং-জয়েন" => return BuiltInFunctionList::_string_join(evaluated_arguments, &mut self.lists),
            "_টাইপ" => return BuiltInFunctionList::_type(evaluated_arguments),
            "_রিড-ফাইল" => return BuiltInFunctionList::_read_file(evaluated_arguments),
            "_রাইট-ফাইল" => return BuiltInFunctionList::_write_file(evaluated_arguments),
            "_ডিলিট-ফাইল" => return BuiltInFunctionList::_delete_file(evaluated_arguments),
            "_নতুন-ডাইরেক্টরি" => return BuiltInFunctionList::_create_dir(evaluated_arguments),
            "_রিড-ডাইরেক্টরি" => {
                // Files also could be dir
                let all_file_names_in_dir = BuiltInFunctionList::_read_dir(evaluated_arguments);
                // Converting vec<string> to vec<datatype>
                let all_file_names = all_file_names_in_dir.iter()
                    .map(|name| DataType::String(name.clone())).collect();

                self.lists.push(all_file_names);
                return DataType::List(self.lists.len() - 1);
            },
            "_ডিলিট-ডাইরেক্টরি" => return BuiltInFunctionList::_delete_dir(evaluated_arguments),
            "_ফাইল-নাকি-ডাইরেক্টরি" => return BuiltInFunctionList::_file_or_dir(evaluated_arguments),
            built_in_function_name => {
                panic!("Built-in function: {} not defined", built_in_function_name)
            },
        }
    }

    fn interpret_func_call_expr(&mut self, f: parser::FunctionCall) -> DataType {
        let env_count_before_fn_call = self.envs.len();

        match *f.expr.clone() {
            parser::Expr::Primary(parser::Primary::Var(func_token)) => {
                //  Checking if function is built-in
                if self.built_in_functions.is_built_in(&func_token.lexeme) {
                    // Function is definitely built-in
                    return self.call_built_in_function(&f, &func_token); // this will return DataType or panics
                } else {
                    // Functions is definitely user-defined and not built-in

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
            parser::Primary::List(array) => {
                // this is a internal pakhi array data type representation
                let mut pakhi_array: Vec<DataType> = Vec::new();
                for elem in array {
                    pakhi_array.push(self.interpret_expr(elem));
                }

                self.lists.push(pakhi_array);
                return DataType::List(self.lists.len() - 1);
            },
            parser::Primary::Group(expr) => self.interpret_expr(*expr),
            parser::Primary::NamelessRecord(key_values) => {
                let mut record: HashMap<String, DataType> = HashMap::new();

                for (i, k) in key_values.0.iter().enumerate() {
                    let key = self.interpret_expr(k.clone());
                    if let DataType::String(string_key) = key {
                        record.insert(string_key, self.interpret_expr(key_values.1[i].clone()));
                    }
                }

                self.nameless_records.push(record);
                return  DataType::NamelessRecord(self.nameless_records.len() - 1);
            }
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
            (DataType::List(ref mut left_arr_i), DataType::List(ref mut right_arr_i)) => {
                let left_arr = self.lists.get(*left_arr_i).unwrap().clone();
                let right_arr = self.lists.get(*right_arr_i).unwrap().clone();
                if addsub_expr.operator == TokenKind::Plus {
                    let mut concatted_arr: Vec<DataType> = Vec::new();
                    for elem in left_arr {
                        concatted_arr.push(elem);
                    }
                    for elem in right_arr {
                        concatted_arr.push(elem);
                    }
                    self.lists.push(concatted_arr);
                    return DataType::List(self.lists.len() - 1);
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
    let mut real_io = RealIO::new();
    let mut interpreter = Interpreter::new(ast, &mut real_io);
    interpreter.run();
}