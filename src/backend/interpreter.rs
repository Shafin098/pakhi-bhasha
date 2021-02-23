use std::collections::HashMap;
use crate::common::io::{IO, RealIO};
use crate::frontend::parser;
use crate::frontend::lexer::{TokenKind, Token};
use crate::backend::built_ins::BuiltInFunctionList;
use crate::backend::mark_sweep;
use crate::common::pakhi_error::PakhiErr;
use std::iter::FromIterator;
use crate::common::pakhi_error::PakhiErr::{RuntimeError, TypeError};

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
    // free list tracks which list indexes are free to be re-used for allocating as list datatype
    free_lists: Vec<usize>,
    nameless_records: Vec<HashMap<String, DataType>>,
    // free records tracks which record indexes are free to be re-used for allocating as record datatype
    free_nameless_records: Vec<usize>,
    // This is used as parameter of gc to decide if it's time to collect garbage
    total_allocated_object_count: usize,
    io: &'a mut T,
    // Storing all built-in function names because when modules identifiers are renamed
    // we don't want to rename built-in functions
    built_in_functions: BuiltInFunctionList,
}

impl<'a, T: 'a + IO> Interpreter<'a, T> {
    pub fn new(statements: Vec<parser::Stmt>, io: &mut T) -> Interpreter<T> {
        Interpreter {
            current: 0,
            statements,
            loops: Vec::new(),
            return_addrs: Vec::new(),
            envs: vec![HashMap::new()],
            previous_if_was_executed: Vec::new(),
            lists: Vec::new(),
            free_lists: Vec::new(),
            nameless_records: Vec::new(),
            free_nameless_records: Vec::new(),
            total_allocated_object_count: 0,
            io,
            built_in_functions: BuiltInFunctionList::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), PakhiErr> {
        loop {
            if let  parser::Stmt::EOS(_, _) = self.statements[self.current] {
                break;
            }
            self.interpret()?;
            if self.total_allocated_object_count >= 1000 {
                let mut gc = mark_sweep::GC::new(&mut self.envs, &mut self.lists,
                                             &mut self.free_lists,
                                             &mut self.nameless_records,
                                             &mut self.free_nameless_records);
                gc.collect_garbage();
                self.total_allocated_object_count = 0;
            }
        }

        Ok(())
    }

    fn interpret(&mut self) -> Result<(), PakhiErr> {
        match self.statements[self.current].clone() {
            parser::Stmt::Print(expr, _, _) => self.interpret_print_stmt(expr)?,
            parser::Stmt::PrintNoEOL(expr, _, _) => self.interpret_print_no_eol(expr)?,
            parser::Stmt::Assignment(assign_stmt, _, _) => self.interpret_assign_stmt(assign_stmt)?,
            parser::Stmt::If(cond_expr, _, _) => self.interpret_if_stmt(cond_expr)?,
            parser::Stmt::Else(_, _) => self.interpret_else_stmt()?,
            parser::Stmt::FuncDef(_, _) => self.interpret_funcdef()?,
            parser::Stmt::Expression(expr, _, _) => {
                self.interpret_expr(expr)?;
                self.current += 1;
            },
            parser::Stmt::Loop(_, _) => {
                // consuming loop
                self.current += 1;

                // saving loop start to reuse in continue statement
                self.loops.push(LoopEnv { start: self.current, total_envs_at_loop_creation: self.envs.len()});

            },
            parser::Stmt::Continue(_, _) => {
                // destroying envs that was created inside loop
                let last_loop_env_index = self.loops.len() - 1;
                let total_envs_created_inside_loop = self.envs.len() - self.loops[last_loop_env_index].total_envs_at_loop_creation;
                for _ in 0..total_envs_created_inside_loop {
                    self.envs.pop();
                }

                let loop_start = self.loops[last_loop_env_index].start;

                self.current = loop_start;
            },
            parser::Stmt::Break(_, _) => {
                self.current += 1;

                // len <= 0 means no new environment was made inside loop
                if self.loops.len() > 0 {
                    // destroying all envs that was created inside loop
                    let last_loop_env_index = self.loops.len() - 1;
                    let total_envs_created_inside_loop = self.envs.len() - self.loops[last_loop_env_index].total_envs_at_loop_creation;
                    for _ in 0..total_envs_created_inside_loop {
                        self.envs.pop();
                    }
                }

                // destroying loop env
                self.loops.pop();

                let mut stack: Vec<char> = Vec::new();
                loop {
                    if let parser::Stmt::Loop(_, _) = self.statements[self.current] {
                        stack.push('{');
                    }

                    if let parser::Stmt::Continue(_, _) = self.statements[self.current] {
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
            parser::Stmt::BlockStart(_, _) => {
                self.current += 1;
                // creating new scope
                self.envs.push(HashMap::new());
            },
            parser::Stmt::BlockEnd(_, _) => {
                self.current += 1;
                // BlockEnd means all statements in this blocks scope were interpreted
                // so destroying scope created by Stmt::BlockStart
                self.envs.pop();
            }
            _ => {
                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                return Err(PakhiErr::RuntimeError(line, file_name,
                              format!("Debug Statement {:#?}", self.statements[self.current])));
            },
        }
        Ok(())
    }

    fn interpret_print_no_eol(&mut self, expr: parser::Expr) -> Result<(), PakhiErr> {
        match self.interpret_expr(expr)? {
            DataType::Num(n) => {
                let num = self.to_bn_num(n)?;
                self.io.print( num.as_str())
            },
            DataType::Bool(b) => self.io.print( self.to_bn_bool(b).as_str()),
            DataType::String(s) => self.io.print(s.as_str()),
            DataType::List(arr_i) => {
                let mut elems: Vec<(usize, DataType)>  = Vec::new();
                for (i, elem) in self.lists[arr_i].iter().enumerate() {
                    elems.push((i, elem.clone()));
                }
                self.io.print("[");
                for (i, elem) in elems {
                    self.print_datatype(elem.clone())?;
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
                    self.print_datatype(v.clone())?;
                    self.io.print(",")
                }
                self.io.print("}");
            },
            _ => {
                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                return Err(PakhiErr::TypeError(line, file_name,
                                               "_দেখাও statement doesn't support this datattype".to_string()));
            },
        }
        self.current += 1;
        Ok(())
    }

    fn print_datatype(&mut self, data: DataType) -> Result<(), PakhiErr> {
        match data {
            DataType::Num(n) => {
                let num = self.to_bn_num(n)?;
                self.io.print( num.as_str());
            },
            DataType::Bool(b) => self.io.print( self.to_bn_bool(b).as_str()),
            DataType::String(s) => self.io.print(s.as_str()),
            DataType::List(arr_i) => {
                let mut elems: Vec<(usize, DataType)>  = Vec::new();
                for (i, elem) in self.lists[arr_i].iter().enumerate() {
                    elems.push((i.clone(), elem.clone()));
                }
                self.io.print("[");
                for (i, elem) in elems.clone() {
                    self.print_datatype(elem.clone())?;
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
                    self.print_datatype(v.clone())?;
                    self.io.print(",")
                }
                self.io.print("}");
            },
            _ => {
                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                return Err(RuntimeError(line, file_name, "দেখাও doesn't support this datatype".to_string()));
            },
        }
        Ok(())
    }

    fn interpret_print_stmt(&mut self, expr: parser::Expr) -> Result<(), PakhiErr> {
        match self.interpret_expr(expr)? {
            DataType::Num(n) => {
                let num = self.to_bn_num(n)?;
                self.io.println(num.as_str())
            },
            DataType::Bool(b) => self.io.println(self.to_bn_bool(b).as_str()),
            DataType::String(s) => self.io.println( s.as_str()),
            DataType::List(arr_i) => {
                let mut elems: Vec<(usize, DataType)>  = Vec::new();
                for (i, elem) in self.lists[arr_i].iter().enumerate() {
                    elems.push((i, elem.clone()));
                }
                self.io.print("[");
                for (i, elem) in elems.clone() {
                    self.print_datatype(elem.clone())?;
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
                    self.print_datatype(v.clone())?;
                    self.io.print(",")
                }
                self.io.println("}");
            },
            _ => {
                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                return Err(PakhiErr::TypeError(line, file_name,
                                 "দেখাও statement doesn't support this datattype".to_string()));
            },
        }
        self.current += 1;
        Ok(())
    }

    fn interpret_assign_stmt(&mut self, assign_stmt: parser::Assignment) -> Result<(), PakhiErr> {
        let var_key: String = assign_stmt.var_name.lexeme.clone().into_iter().collect();

        match assign_stmt.kind {
            parser::AssignmentKind::FirstAssignment => self.create_new_var(var_key, assign_stmt)?,
            parser::AssignmentKind::Reassignment => self.reassign_to_old_var(var_key, assign_stmt)?,
        }

        self.current += 1;
        Ok(())
    }

    fn create_new_var(&mut self, var_key: String, assign_stmt: parser::Assignment) -> Result<(), PakhiErr>
    {
        match assign_stmt.init_value {
            Some(expr) => {
                let init_value = self.interpret_expr(expr)?;

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
        Ok(())
    }

    fn reassign_to_old_var(&mut self, var_key: String,
                           assign_stmt: parser::Assignment) -> Result<(), PakhiErr>
    {
        let init_expr = assign_stmt.init_value.clone().unwrap();
        let init_value = self.interpret_expr(init_expr)?;

        // if variable wasn't found it evaluates to any negative number
        let var_found_at_env_index: i32 = self.find_var_env_index(var_key.clone(), assign_stmt.init_value.clone());

        if var_found_at_env_index >= 0 {
            if assign_stmt.indexes.is_empty() {
                // only simple variable assignment
                self.envs[var_found_at_env_index as usize].insert(var_key, Some(init_value));
            } else {
                // assignment to element in a list or record
                self.reassign_to_list_or_record(assign_stmt, var_key, var_found_at_env_index, init_value)?;
            }
        } else {
            let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
            return Err(RuntimeError(line, file_name, format!("Variable wasn't declared {:#}", var_key)));
        }
        Ok(())
    }

    // Reassign a value to a list or record at provided index
    fn reassign_to_list_or_record(&mut self,
                                  assign_stmt: parser::Assignment,
                                  var_key: String,
                                  var_found_at_env_index: i32,
                                  init_value: DataType) -> Result<(), PakhiErr>
    {
        // effective_index is index of deepest nested array, to which init_val will be assigned
        let effective_index = self.interpret_expr(assign_stmt.indexes.last().unwrap().clone())?;
        let evaluated_indexes: Vec<Index> = self.evaluate_all_indexes(assign_stmt.indexes.clone())?;

        let var = self.get_var_from_env(var_key.as_str(), var_found_at_env_index as usize);

        match var {
            Some(DataType::List(i)) => {
                if assign_stmt.indexes.len() == 1 {
                    // single dimensional list
                    // changing list element at only one level deep
                    self.list_single_dim_assign(i, effective_index, init_value)?;
                } else {
                    // multidimensional array so need to traverse nested list ore record
                    self.list_multi_dim_assign(i, evaluated_indexes, init_value.clone())?;
                }
            },
            Some(DataType::NamelessRecord(record_ref)) => {
                if assign_stmt.indexes.len() == 1 {
                    // single dimensional list
                    // changing list element at only one level deep
                    self.record_single_dim_assign(record_ref, effective_index, init_value)?;
                } else {
                    // multidimensional array so need to traverse nested list ore record
                    self.record_multi_dim_assign(record_ref, evaluated_indexes, init_value.clone())?;
                }
            },
            _ => {
                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                return Err(TypeError(line, file_name, "Datatype doesn't support index assignment".to_string()));
            },
        }
        Ok(())
    }

    fn list_single_dim_assign(&mut self,
                              list_ref: usize,
                              index: DataType,
                              init_value: DataType) -> Result<(), PakhiErr>
    {
        match index {
            DataType::List(j) => {
                let a = self.lists[j].clone();
                match a[0].clone() {
                    DataType::Num(n) => {
                        let list = self.lists.get_mut(list_ref).unwrap();
                        list[n as usize] = init_value
                    },
                    _ => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name,
                                         "List must be indexed with number type".to_string()));
                    },
                }
            },
            _ => {
                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                return Err(RuntimeError(line, file_name,
                                        "Unexpected error while assigning to a index".to_string()));
            },
        }
        Ok(())
    }

    fn record_single_dim_assign(&mut self,
                                record_ref: usize,
                                index: DataType,
                                init_value: DataType) -> Result<(), PakhiErr>
    {
        match index {
            DataType::List(j) => {
                let a = self.lists[j].clone();
                match a[0].clone() {
                    DataType::String(key) => {
                        let record = self.nameless_records
                                                                .get_mut(record_ref).unwrap();
                        record.insert(key, init_value);
                    },
                    _ => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name,
                                                "Records must be indexed by a string type".to_string()));
                    },
                }
            },
            _ => {
                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                return Err(RuntimeError(line, file_name,
                                        "Unexpected error while assigning to a index".to_string()));
            },
        }
        Ok(())
    }

    fn list_multi_dim_assign(&mut self,
                             list_reference: usize,
                             evaluated_indexes: Vec<Index>,
                             init_value: DataType) -> Result<(), PakhiErr>
    {
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
                                        break;
                                    },
                                    _ => {
                                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                                        return Err(RuntimeError(line, file_name, "Error on assignment to index".to_string()));
                                    }
                                }
                            }
                            _ => {
                                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                                return Err(RuntimeError(line, file_name, "Cannot assign at index if datatype is not list".to_string()));
                            },
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
                                    _ => {
                                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                                        return Err(RuntimeError(line, file_name, "Error on assignment to index".to_string()));
                                    },
                                }
                            },
                            _ => {
                                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                                return Err(RuntimeError(line, file_name, "Cannot index if datatype not list".to_string()));
                            },
                        }
                    }
                }
            },
            _ => {
                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                return Err(RuntimeError(line, file_name, "Only list and record datatype can be indexed".to_string()));
            },
        }

        Ok(())
    }

    fn record_multi_dim_assign(&mut self,
                               record_reference: usize,
                               evaluated_indexes: Vec<Index>,
                               init_value: DataType) -> Result<(), PakhiErr>
    {
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
                                    _ => {
                                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                                        return Err(RuntimeError(line, file_name,
                                                     "Error on assignment to index".to_string()));
                                    }
                                }
                            }
                            _ => {
                                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                                return Err(RuntimeError(line, file_name,
                                   "Cannot assign at index if datatype is not record".to_string()));
                            },
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
                                    _ => {
                                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                                        return Err(RuntimeError(line, file_name,
                                     "Cannot assign at index if datatype is not record".to_string()));
                                    },
                                }
                            }
                            _ => {
                                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                                return Err(RuntimeError(line, file_name, "Cannot assign at index if datatype is not record".to_string()));
                            },
                        }
                    }
                }
            }
            _ => {
                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                return Err(RuntimeError(line, file_name, "Only list and record datatype can be indexed".to_string()));
            },
        }

        Ok(())
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

    fn evaluate_all_indexes(&mut self, index_exprs: Vec<parser::Expr>) -> Result<Vec<Index>, PakhiErr> {
        let mut evaluated_index_exprs: Vec<Index> = Vec::new();

        for i in 0..index_exprs.len() {
            let index = self.interpret_expr(index_exprs[i].clone())?;
            match  index {
                DataType::List(arr_i) => {
                    match self.lists[arr_i][0].clone() {
                        DataType::Num(i) => evaluated_index_exprs.push(Index::List(i as usize)),
                        DataType::String(key) => evaluated_index_exprs.push(Index::NamelessRecord(key)),
                        _ => {
                            let (line, file_name) = self.extract_expr_err_meta(&index_exprs[i]);
                            return Err(RuntimeError(line, file_name, "Index must be of number or string type".to_string()));
                        },
                    }
                }
                _ => {
                    let (line, file_name) = self.extract_expr_err_meta(&index_exprs[i]);
                    return Err(RuntimeError(line, file_name, "Expected '[' for indexing".to_string()));
                },
            }
        }

        return  Ok(evaluated_index_exprs);
    }

    fn interpret_funcdef(&mut self) -> Result<(), PakhiErr> {
        // consuming function definition statement
        self.current += 1;

        if let parser::Stmt::Expression(parser::Expr::Call(function, _, _),
                                        line, file_name) = self.statements[self.current].clone()
        {
            match *function.expr {
                parser::Expr::Primary(parser::Primary::Var(func_token), line, file_name) => {
                    let func_name: String = func_token.lexeme.iter().collect();
                    let func_args = function.arguments;
                    let mut func_args_name: Vec<String> = Vec::new();

                    for arg_expr in func_args {
                        match arg_expr {
                            parser::Expr::Primary(parser::Primary::Var(name_token), _, _) => {
                                func_args_name.push(name_token.lexeme.iter().collect());
                            },
                            _ => {
                                return Err(RuntimeError(line, file_name, "Error during function definition".to_string()));
                            },
                        }
                    }

                    let func = Func {
                        starting_statement: self.current + 1,
                        args: func_args_name,
                    };

                    let current_env_i = self.envs.len() - 1;
                    self.envs[current_env_i].insert(func_name.clone(), Some(DataType::Function(func)));
                },
                _ => {
                    return Err(RuntimeError(line, file_name, "Cannot interpret function definition".to_string()));
                },
            }
        } else {
            let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
            return Err(RuntimeError(line, file_name, "Expected function definition".to_string()));
        }

        // consuming function name and args statement (Expr::Call)
        self.current += 1;

        // skipping all statements in function body
        // statements in func body is not executed during func definition
        self.skip_block()?;

        // consuming return statement
        if self.current >= self.statements.len() {
            let (line, file_name) = self.extract_err_meta_stmt(self.statements.len() - 1)?;
            return Err(RuntimeError(line, file_name, "Unexpected error at function call".to_string()));
        }
        if let parser::Stmt::Return(_, _, _) = self.statements[self.current].clone() {
           self.current += 1;
        } else {
            let (line, file_name) = self.extract_err_meta_stmt(self.statements.len() - 1)?;
            return Err(RuntimeError(line, file_name, "Expected a return statement".to_string()));
        }

        Ok(())
    }

    fn interpret_if_stmt(&mut self, expr: parser::Expr) -> Result<(), PakhiErr> {
        let (line, file_name) = self.extract_expr_err_meta(&expr);

        // consuming if token
        self.current += 1;

        let if_condition_expr = self.interpret_expr(expr)?;

        if let DataType::Bool(condition) = if_condition_expr {
            if condition == false {
                self.previous_if_was_executed.push(false);
                // condition expression of if statement is false so skipping next block statement
                self.skip_block_in_if()?;
            } else {
                self.previous_if_was_executed.push(true);
            }
        } else {
            return Err(RuntimeError(line, file_name,
                      "If condition expression must evaluate to boolean value".to_string()));
        }

        Ok(())
    }

    fn interpret_else_stmt(&mut self) -> Result<(), PakhiErr> {
        assert!(!self.previous_if_was_executed.is_empty());

        // consuming else token
        self.current += 1;

        let last_if_condition_index = self.previous_if_was_executed.len() - 1;
        if self.previous_if_was_executed[last_if_condition_index] {
            self.skip_block_in_if()?;
        }
        self.previous_if_was_executed.pop();

        Ok(())
    }

    fn skip_block(&mut self) -> Result<(), PakhiErr> {
        let mut stack: Vec<char> = Vec::new();

        while self.current < self.statements.len() {
            if let parser::Stmt::BlockStart(_, _) = self.statements[self.current] {
                stack.push('{');
            }

            if let parser::Stmt::BlockEnd(_, _) = self.statements[self.current] {
                let previous = stack.pop();
                match previous {
                    Some(_) => {
                        if stack.is_empty() {
                            // consuming Stmt::BlockEnd
                            self.current += 1;
                            break;
                        }
                    },
                    None => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, "Error during function call".to_string()));
                    },
                }
            }

            // skipping statements in block
            self.current += 1;
        }

        Ok(())
    }

    fn skip_block_in_if(&mut self) -> Result<(), PakhiErr> {
        self.skip_block()?;

        match self.statements[self.current] {
            parser::Stmt::Else(_, _) => {},
            _ => { self.previous_if_was_executed.pop(); },
        }

        Ok(())
    }

    fn interpret_expr(&mut self, expr: parser::Expr) -> Result<DataType, PakhiErr> {
        match expr {
            parser::Expr::Primary(p, _, _) => {
                let expr = self.interpret_primary_expr(p)?;
                return Ok(expr);
            },
            parser::Expr::Unary(u_expr, _, _) => {
                let expr = self.interpret_unary_expr(u_expr)?;
                return Ok(expr);
            },
            parser::Expr::And(and_expr, _, _) => {
                let expr = self.interpret_and_expr(and_expr)?;
                return Ok(expr);
            },
            parser::Expr::Or(or_expr, _, _) => {
                let expr = self.interpret_or_expr(or_expr)?;
                return Ok(expr);
            },
            parser::Expr::Equality(eq_expr, _, _) => {
                let expr = self.interpret_eq_expr(eq_expr)?;
                return Ok(expr);
            },
            parser::Expr::Comparison(comp_expr, _, _) => {
                let expr = self.interpret_comp_expr(comp_expr)?;
                return Ok(expr);
            },
            parser::Expr::AddOrSub(addsub_expr, _, _) => {
                let expr = self.interpret_addsub_expr(addsub_expr)?;
                return Ok(expr);
            },
            parser::Expr::MulOrDivOrRemainder(muldiv_expr, _, _) => {
                let expr = self.interpret_muldiv_remainder_expr(muldiv_expr)?;
                return Ok(expr);
            },
            parser::Expr::Call(function, _, _) => {
                let expr = self.interpret_func_call_expr(function)?;
                return Ok(expr);
            },
            parser::Expr::Indexing(identifier, i, _, _) => {
                let expr = self.interpret_indexing(identifier, i)?;
                return Ok(expr);
            },
        }
    }

    fn interpret_indexing(&mut self,
                          identifier_expr: Box<parser::Expr>,
                          index_expr: Box<parser::Expr>) -> Result<DataType, PakhiErr>
    {
        let (line, file_name) = self.extract_expr_err_meta(&*index_expr);

        let identifier = self.interpret_expr(*identifier_expr)?;
        let index = self.interpret_expr(*index_expr)?;

        match (identifier, index) {
            (DataType::List(arr_i), DataType::Num(i)) => {
                let arr = self.lists[arr_i].clone();
                return Ok(arr[i as usize].clone());
            },
            (DataType::NamelessRecord(record_i), DataType::String(key)) => {
                let nameless_record = self.nameless_records[record_i].clone();
                let record_data = nameless_record.get(&*key).unwrap().clone();
                return Ok(record_data);
            },
            (_, DataType::Num(_)) => {
                return Err(RuntimeError(line, file_name, "Only list supports indexing with number".to_string()));
            },
            (DataType::List(_), _) => {
                return Err(TypeError(line, file_name, "List index must of number type".to_string()));
            },
            (_, DataType::String(_)) => {
                return Err(RuntimeError(line, file_name, "Only record supports indexing with string".to_string()));
            },
            (DataType::NamelessRecord(_), _) => {
                return Err(TypeError(line, file_name, "Record index must be of string type".to_string()));
            },
            _ => {
                return Err(RuntimeError(line, file_name, "Invalid indexing format".to_string()));
            },
        }
    }

    fn call_built_in_function(&mut self, f: &parser::FunctionCall, func_token: &Token) -> Result<DataType, PakhiErr> {
        let mut evaluated_arguments: Vec<DataType> = Vec::new();
        // Evaluating all arguments
        for arg in f.arguments.iter() {
            let e_a = self.interpret_expr(arg.clone())?;
            evaluated_arguments.push(e_a);
        }
        // Finding out which built-in function and executing that accordingly
        match self.built_in_functions.get_name(&func_token.lexeme).as_str() {
            "_স্ট্রিং" => {
                match BuiltInFunctionList::_to_string(evaluated_arguments) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_সংখ্যা" => {
                match  BuiltInFunctionList::_to_num(evaluated_arguments) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_লিস্ট-পুশ" => {
                match BuiltInFunctionList::_list_push(evaluated_arguments, &mut self.lists) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_লিস্ট-পপ" => {
                match BuiltInFunctionList::_list_pop(evaluated_arguments, &mut self.lists) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_লিস্ট-লেন" => {
                match BuiltInFunctionList::_list_len(evaluated_arguments, &mut self.lists) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_রিড-লাইন" => {
                match BuiltInFunctionList::_read_line(evaluated_arguments) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_এরর" => {
                let call_result = BuiltInFunctionList::_error(evaluated_arguments);
                let err_m: String;
                match call_result {
                    Ok(error) => err_m = error,
                    Err(error) => err_m = error,
                }
                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                return Err(RuntimeError(line, file_name, err_m));
            },
            "_স্ট্রিং-স্প্লিট" => {
                match BuiltInFunctionList::_string_split(evaluated_arguments, &mut self.lists) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_স্ট্রিং-জয়েন" => {
                match BuiltInFunctionList::_string_join(evaluated_arguments, &mut self.lists) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_টাইপ" => {
                match BuiltInFunctionList::_type(evaluated_arguments) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_রিড-ফাইল" => {
                match BuiltInFunctionList::_read_file(evaluated_arguments) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_রাইট-ফাইল" => {
                match BuiltInFunctionList::_write_file(evaluated_arguments) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_ডিলিট-ফাইল" => {
                match BuiltInFunctionList::_delete_file(evaluated_arguments) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            }
            "_নতুন-ডাইরেক্টরি" => {
                match BuiltInFunctionList::_create_dir(evaluated_arguments) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            }
            "_রিড-ডাইরেক্টরি" => {
                // Files also could be dir
                let call_result = BuiltInFunctionList::_read_dir(evaluated_arguments);
                match call_result {
                    Ok(all_file_names_in_dir) => {
                        // Converting vec<string> to vec<datatype>
                        let all_file_names = all_file_names_in_dir.iter()
                            .map(|name| DataType::String(name.clone())).collect();

                        let pakhi_list_data = self.create_new_list_datatype(all_file_names);
                        return Ok(pakhi_list_data);
                    },
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_ডিলিট-ডাইরেক্টরি" => {
                match BuiltInFunctionList::_delete_dir(evaluated_arguments) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            },
            "_ফাইল-নাকি-ডাইরেক্টরি" => {
                match BuiltInFunctionList::_file_or_dir(evaluated_arguments) {
                    Ok(result_data) => Ok(result_data),
                    Err(err) => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        return Err(RuntimeError(line, file_name, err));
                    }
                }
            }
            built_in_function_name => {
                return Err(RuntimeError(func_token.line, func_token.clone().src_file_path,
                          format!("Built-in function: {} not defined", built_in_function_name)));

            },
        }
    }

    fn interpret_func_call_expr(&mut self, f: parser::FunctionCall) -> Result<DataType, PakhiErr> {
        let env_count_before_fn_call = self.envs.len();

        match *f.expr.clone() {
            parser::Expr::Primary(parser::Primary::Var(func_token), _, _) => {
                //  Checking if function is built-in
                if self.built_in_functions.is_built_in(&func_token.lexeme) {
                    // Function is definitely built-in
                    return self.call_built_in_function(&f, &func_token); // this will return DataType or panic)
                } else {
                    // These are for error reporting
                    let line = func_token.line;
                    let src_path = func_token.src_file_path.clone();
                    let func_name = String::from_iter(func_token.lexeme.iter());

                    // Functions is definitely user-defined and not built-in

                    // this block checks if function was declared,
                    // sets up environment, inserts args to new environment
                    // and saves return address for function call
                    let func = self.interpret_var(func_token)?;

                    if let DataType::Function(func) = func {
                        let mut root_env: HashMap<String, Option<DataType>> = HashMap::new();
                        for i in 0..func.args.len() {
                            if i < f.arguments.len() {
                                let arg = self.interpret_expr(f.arguments[i].clone())?;
                                root_env.insert(func.args[i].clone(), Option::from(arg));
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
                        return Err(RuntimeError(line, src_path,
                                                format!("Function '{}' not Declared", func_name)));
                    }
                }

            },
            _ => {
                let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                return Err(RuntimeError(line, file_name, "Calling undefined function".to_string()));
            },
        }

        // jumping to function start and starting executing statements in function body

        match &self.statements[self.current] {
            parser::Stmt::BlockStart(_, _) => {},
            // TODO show file name and line number by matching all enum variant
            _ => self.io.panic(PakhiErr::UnexpectedError("Expected '{'".to_string())),
        }

        // assert_eq!(parser::Stmt::BlockStart, self.statements[self.current]);
        // interpreting all statements inside function body
        // assuming self.current was set at function start
        loop {
            if let parser::Stmt::Return(_, _, _) = self.statements[self.current].clone() {
                break;
            } else {
                self.interpret()?;
            }
        }

        if let parser::Stmt::Return(expr, _, _) = self.statements[self.current].clone() {
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

        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
        return Err(RuntimeError(line, file_name, "Error calling function".to_string()));
    }

    fn interpret_primary_expr(&mut self, p: parser::Primary) -> Result<DataType, PakhiErr> {
        match p {
            parser::Primary::Nil => return Ok(DataType::Nil),
            parser::Primary::String(s) => return Ok(DataType::String(s)),
            parser::Primary::Num(n) => return Ok(DataType::Num(n)),
            parser::Primary::Bool(b) => return Ok(DataType::Bool(b)),
            parser::Primary::Var(v) => {
                let var = self.interpret_var(v)?;
                return Ok(var);
            },
            parser::Primary::List(array) => {
                // this is a internal pakhi array data type representation
                let mut pakhi_array: Vec<DataType> = Vec::new();
                for elem in array {
                    let evaluated_elem = self.interpret_expr(elem)?;
                    pakhi_array.push(evaluated_elem);
                }
                let pakhi_list_data = self.create_new_list_datatype(pakhi_array);
                return Ok(pakhi_list_data);
            },
            parser::Primary::Group(expr) => {
                let expr = self.interpret_expr(*expr)?;
                return Ok(expr);
            },
            parser::Primary::NamelessRecord(key_values) => {
                let mut record: HashMap<String, DataType> = HashMap::new();

                for (i, k) in key_values.0.iter().enumerate() {
                    let key = self.interpret_expr(k.clone())?;
                    if let DataType::String(string_key) = key {
                        let new_val = self.interpret_expr(key_values.1[i].clone())?;
                        record.insert(string_key, new_val);
                    }
                }

                let pakhi_record_datatype = self.create_new_nameless_record_datatype(record);
                return Ok(pakhi_record_datatype);
            }
        }
    }

    fn interpret_unary_expr(&mut self, u_expr: parser::Unary) -> Result<DataType, PakhiErr> {
        let (line, file_name) = self.extract_expr_err_meta(&u_expr.right.clone());

        let expr_val = self.interpret_expr(*u_expr.right)?;
        match expr_val {
            DataType::Num(n) => {
                if u_expr.operator == TokenKind::Minus {
                    return Ok(DataType::Num(n * -1.0));
                }
                return Err(TypeError(line, file_name, "Unsupported '-' unary operation on type".to_string()));
            },
            DataType::Bool(b) => {
                if u_expr.operator == TokenKind::Not {
                    return Ok(DataType::Bool(!b));
                }
                return Err(TypeError(line, file_name, "Unsupported '!' unary operation on type".to_string()));
            },
            _ => {
                return Err(TypeError(line, file_name, "Datatype doesn't support unary operation".to_string()));
            }
        }
    }

    fn interpret_and_expr(&mut self, and_expr: parser::And) -> Result<DataType, PakhiErr> {
        let (line, file_name) = self.extract_expr_err_meta(&and_expr.left.clone());

        let right_expr_val = self.interpret_expr(*and_expr.right)?;
        let left_expr_val = self.interpret_expr(*and_expr.left)?;

        match (right_expr_val, left_expr_val) {
            (DataType::Bool(right), DataType::Bool(left)) => return Ok(DataType::Bool(right && left)),
            (DataType::Bool(_), _) => {
                return Err(TypeError(line, file_name, "Datatype doesn't support and operation".to_string()));
            },
            (_, DataType::Bool(_)) => {
                return Err(TypeError(line, file_name, "Datatype doesn't support and operation".to_string()));
            }
            _ => {
                return Err(TypeError(line, file_name, "Datatype doesn't support and operation".to_string()));
            }
        }
    }

    fn interpret_or_expr(&mut self, or_expr: parser::Or) -> Result<DataType, PakhiErr> {
        let (line, file_name) = self.extract_expr_err_meta(&or_expr.left.clone());

        let right_expr_val = self.interpret_expr(*or_expr.right)?;
        let left_expr_val = self.interpret_expr(*or_expr.left)?;

        match (right_expr_val, left_expr_val) {
            (DataType::Bool(right), DataType::Bool(left)) => return Ok(DataType::Bool(right || left)),
            (DataType::Bool(_), _) => {
                return Err(TypeError(line, file_name, "Datatype doesn't support or operation".to_string()));
            },
            (_, DataType::Bool(_)) => {
                return Err(TypeError(line, file_name, "Datatype doesn't support or operation".to_string()));
            }
            _ => {
                return Err(TypeError(line, file_name, "Datatype doesn't support or operation".to_string()));
            }
        }
    }

    fn interpret_addsub_expr(&mut self, addsub_expr: parser::Binary) -> Result<DataType, PakhiErr> {
        let (line, file_name) = self.extract_expr_err_meta(&*addsub_expr.left.clone());

        let left_expr_val = self.interpret_expr(*addsub_expr.left)?;
        let right_expr_val = self.interpret_expr(*addsub_expr.right)?;

        match (left_expr_val, right_expr_val) {
            (DataType::Num(left), DataType::Num(right)) => {
                match addsub_expr.operator {
                    TokenKind::Plus => return Ok(DataType::Num(left + right)),
                    TokenKind::Minus => return Ok(DataType::Num(left - right)),
                    _ => {
                        return Err(TypeError(line, file_name, "Invalid operation number type".to_string()));
                    },
                }
            },
            (DataType::String(left_str), DataType::String(right_str)) => {
                if addsub_expr.operator == TokenKind::Plus {
                    return Ok(DataType::String(format!("{}{}", left_str, right_str)));
                }

                return Err(TypeError(line, file_name, "Invalid operation string type".to_string()));
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
                    let pakhi_list_data = self.create_new_list_datatype(concatted_arr);
                    return Ok(pakhi_list_data);
                }

                return Err(TypeError(line, file_name, "Invalid operation list type".to_string()));
            }
            _ => {
                return Err(TypeError(line, file_name,
                            "Invalid operation, operand doesn't support this operation".to_string()));
            },
        }
    }

    fn interpret_muldiv_remainder_expr(&mut self, muldiv_expr: parser::Binary) -> Result<DataType, PakhiErr> {
        let (line, file_name) = self.extract_expr_err_meta(&*muldiv_expr.left);

        let right_expr_val = self.interpret_expr(*muldiv_expr.right)?;
        let left_expr_val = self.interpret_expr(*muldiv_expr.left)?;

        if let DataType::Num(right)  = right_expr_val {
            if let DataType::Num(left) = left_expr_val {
                match muldiv_expr.operator {
                    TokenKind::Multiply => return Ok(DataType::Num(left * right)),
                    TokenKind::Division => return Ok(DataType::Num(left / right)),
                    TokenKind::Remainder => return Ok(DataType::Num(left % right)),
                    _ => {
                        return Err(TypeError(line, file_name, "Type doesn't support operation".to_string()));
                    },
                }
            }
        }

        return Err(TypeError(line, file_name, "Type doesn't support operation".to_string()));
    }

    fn interpret_eq_expr(&mut self, eq_expr: parser::Binary) -> Result<DataType, PakhiErr> {
        let (line, file_name) = self.extract_expr_err_meta(&*eq_expr.left.clone());

        let evaluated_left_expr = self.interpret_expr(*eq_expr.left)?;
        let evaluated_right_expr = self.interpret_expr(*eq_expr.right)?;

        match eq_expr.operator {
            TokenKind::EqualEqual => {
                return Ok(DataType::Bool(evaluated_left_expr == evaluated_right_expr));
            },
            TokenKind::NotEqual =>  {
                return Ok(DataType::Bool(evaluated_left_expr != evaluated_right_expr ));
            },
            _ => {
                return Err(TypeError(line, file_name, "Type doesn't support operation".to_string()));
            }
        }
    }

    fn interpret_comp_expr(&mut self, comp_expr: parser::Binary) -> Result<DataType, PakhiErr> {
        let (line, file_name) = self.extract_expr_err_meta(&*comp_expr.left.clone());

        let evaluated_left_expr = self.interpret_expr(*comp_expr.left)?;
        let evaluated_right_expr = self.interpret_expr(*comp_expr.right)?;

        match (evaluated_left_expr.clone(), evaluated_right_expr.clone()) {
            (DataType::Num(_), DataType::Num(_)) => {
                match comp_expr.operator {
                    TokenKind::GreaterThan => {
                        return Ok(DataType::Bool(evaluated_left_expr > evaluated_right_expr));
                    },
                    TokenKind::GreaterThanOrEqual => {
                        return Ok(DataType::Bool(evaluated_left_expr >= evaluated_right_expr));
                    },
                    TokenKind::LessThan => {
                        return Ok(DataType::Bool(evaluated_left_expr < evaluated_right_expr));
                    },
                    TokenKind::LessThanOrEqual => {
                        return Ok(DataType::Bool(evaluated_left_expr <= evaluated_right_expr));
                    },
                    _ => {
                        return Err(TypeError(line, file_name, "Type doesn't support operation".to_string()));
                    }
                }
            },
            _ => {
                return Err(TypeError(line, file_name, "Type doesn't support operation".to_string()));
            }
        }
    }

    fn interpret_var(&mut self, v: Token) -> Result<DataType, PakhiErr> {
        let var_key: String = v.lexeme.clone().into_iter().collect();

        for env in self.envs.iter_mut().rev() {
            let expr_result = env.get(&*var_key);
            if expr_result.is_some() {
                match expr_result.unwrap() {
                    Some(var_value) => return Ok(var_value.clone()),
                    None => {
                        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                        let var_name = String::from_iter(v.lexeme.iter());
                        return Err(PakhiErr::RuntimeError(line, file_name,
                                              format!("Variable wasn't initialized {}", var_name)));
                    },
                }
            }
        }
        let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
        let var_name = String::from_iter(v.lexeme.iter());
        return Err(PakhiErr::RuntimeError(line, file_name,
                                          format!("Variable wasn't initialized {}", var_name)));
    }

    fn create_new_list_datatype(&mut self, new_list: Vec<DataType>) -> DataType {
        // self.total_allocated_object_count is used as a parameter in gc to determine
        // if its time collect garbage
        self.total_allocated_object_count += new_list.len();

        if self.free_lists.len() > 0 {
            let free_index = self.free_lists.pop().unwrap();
            self.lists[free_index] = new_list;
            return DataType::List(free_index);
        } else {
            self.lists.push(new_list);
            return DataType::List(self.lists.len() - 1);
        }
    }

    fn create_new_nameless_record_datatype(&mut self, new_record: HashMap<String, DataType>) -> DataType {
        // self.total_allocated_object_count is used as a parameter in gc to determine
        // if its time collect garbage
        self.total_allocated_object_count += new_record.len();

        if self.free_nameless_records.len() > 0 {
            let free_index = self.free_nameless_records.pop().unwrap();
            self.nameless_records[free_index] = new_record;
            return DataType::NamelessRecord(free_index);
        } else {
            self.nameless_records.push(new_record);
            return DataType::NamelessRecord(self.nameless_records.len() - 1);
        }
    }

    fn extract_err_meta_stmt(&self, i: usize) -> Result<(u32, String), PakhiErr> {
        if self.current >= self.statements.len() {
            return Err(PakhiErr::UnexpectedError("Unexpected error, probably missing ';'".to_string()));
        } else {
            match &self.statements[i] {
                parser::Stmt::Print(_, line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::PrintNoEOL(_, line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::Assignment(_, line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::Expression(_, line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::BlockStart(line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::BlockEnd(line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::FuncDef(line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::Return(_, line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::If(_, line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::Loop(line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::Continue(line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::Break(line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::Else(line, file_name) => Ok((line.clone(), file_name.clone())),
                parser::Stmt::EOS(line, file_name) => Ok((line.clone(), file_name.clone())),
            }
        }
    }

    fn extract_expr_err_meta(&self, expr: &parser::Expr) -> (u32, String) {
        match expr {
            parser::Expr::Indexing(_, _, line , file_name) => (line.clone(), file_name.clone()),
            parser::Expr::Or(_, line, file_name) => (line.clone(), file_name.clone()),
            parser::Expr::And(_, line, file_name) => (line.clone(), file_name.clone()),
            parser::Expr::Equality(_, line, file_name) => (line.clone(), file_name.clone()),
            parser::Expr::Comparison(_, line, file_name) => (line.clone(), file_name.clone()),
            parser::Expr::AddOrSub(_, line, file_name) => (line.clone(), file_name.clone()),
            parser::Expr::MulOrDivOrRemainder(_, line, file_name) => (line.clone(), file_name.clone()),
            parser::Expr::Unary(_, line, file_name) => (line.clone(), file_name.clone()),
            parser::Expr::Call(_, line, file_name) => (line.clone(), file_name.clone()),
            parser::Expr::Primary(_, line, file_name) => (line.clone(), file_name.clone()),
       }
    }

    fn to_bn_num(&self, n: f64) -> Result<String, PakhiErr> {
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
               _ => {
                   let (line, file_name) = self.extract_err_meta_stmt(self.current)?;
                   return Err(RuntimeError(line, file_name,
                    format!("Cannot convert '{}' to number", digit)));
               },
           }
        }

        Ok(bangla_num_string)
    }

    fn to_bn_bool(&self, b: bool) -> String {
        match b {
            true => "সত্য".to_string(),
            false => "মিথ্যা".to_string(),
        }
    }
}

pub fn run(ast: Vec<parser::Stmt>) -> Result<(), PakhiErr> {
    let mut real_io = RealIO::new();
    let mut interpreter = Interpreter::new(ast, &mut real_io);
    return interpreter.run();
}