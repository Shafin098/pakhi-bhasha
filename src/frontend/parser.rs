use crate::frontend::lexer;
use crate::frontend::lexer::Token;
use crate::frontend::lexer::TokenKind;
use crate::common::io;
use crate::common::io::IO;
use std::path::Path;
use std::collections::HashMap;
use std::ffi::OsStr;

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
    // assignment could me made to list or record element, so indexes are needed
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
    Indexing(Box<Expr>, Box<Expr>),
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
    main_module_path: String,
    child_modules: HashMap<String, Vec<String>>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
            main_module_path: String::new(),
            child_modules: HashMap::new(),
        }
    }

    fn parse(&mut self) -> Vec<Stmt> {
        let parent_module_file_name = self.extract_filename(&self.main_module_path);
        let child_modules_paths = self.extract_all_import_paths(&self.tokens);
        let child_modules_file_name = self.extract_filenames(&child_modules_paths);
        let mut new_childs: Vec<String> = Vec::new();
        for new_child_name in child_modules_file_name {
            new_childs.push(new_child_name);
        }
        self.child_modules.insert(parent_module_file_name.clone(), new_childs);

        let mut statements: Vec<Stmt> = Vec::new();
        for _ in 0..self.tokens.len() {
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

        statements
    }

    fn statements(&mut self) -> Stmt {
        match self.tokens[self.current].kind {
            TokenKind::Print => self.print_stmt(),
            TokenKind::PrintNoEOL => self.print_no_newline_stmt(),
            TokenKind::Var => self.assignment_stmt(),
            TokenKind::Identifier => self.re_assignment_or_func_call_stmt(),
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
            TokenKind::Comment => self.comment_block(),
            TokenKind::Import => self.module_import_stmt(),
            TokenKind::EOT => { Stmt::EOS },
             _ => panic!("Err at line: {}\nDebug token{:#?}",
                        self.tokens[self.current].line, self.tokens[self.current]),
        }
    }

    fn module_import_stmt(&mut self) -> Stmt {
        // skipping module keyword token
        self.current += 1;

        if self.tokens[self.current].kind == TokenKind::Identifier {
            let module_import_name = self.tokens[self.current].lexeme.clone();
            self.named_module_import(module_import_name);
        } else {
            panic!("Error at line: {}, Expected a name for imported module", self.tokens[self.current].line)
        }

        // skipping ; token
        self.current += 1;

        // Module doesn't generate statement, it only lexes and puts returned tokens to parser's token
        // queue. Then generates statement from those tokens. That's why self.statements() is called.
        return self.statements();
    }

    // Module could be imported with giving a namespace which was called unnamed_module_import
    // but unnamed module import feature was removed
    // that's why this functions name is named_module_import instead of import_module
    fn named_module_import(&mut self, module_import_name: Vec<char>) {
        // skipping module name identifier token and equal token
        self.current += 2;

        let imported_tokens;
        if let TokenKind::String(module_path) = self.tokens[self.current].kind.clone() {
            // checking if importing file with .pakhi
            if !module_path.ends_with(".pakhi") {
                panic!("Error at line: {} not a valid module file name", self.tokens[self.current].line);
            }
            // skipping module path string token
            self.current += 1;

            imported_tokens = self.get_tokens_from_module(&module_path, module_import_name);
            let parent_module_file_name = self.extract_filename(&module_path);
            let child_modules_paths = self.extract_all_import_paths(&imported_tokens);
            let child_modules_file_name = self.extract_filenames(&child_modules_paths);

            match self.child_modules.get_mut(&*parent_module_file_name) {
                Some(childs) => {
                    for new_child_name in child_modules_file_name {
                        if childs.contains(&new_child_name) {
                            panic!("Cyclic module dependency. \
                                  Can't import {} from {}",parent_module_file_name, new_child_name);
                        }
                        childs.push(new_child_name);
                    }
                },
                None => {
                    let mut new_childs: Vec<String> = Vec::new();
                    for new_child_name in child_modules_file_name {
                        new_childs.push(new_child_name);
                    }
                    self.child_modules.insert(parent_module_file_name.clone(), new_childs);
                }
            }

        } else {
            panic!("Error at line: {}, Expected module import path", self.tokens[self.current].line);
        }

        // tokens is inserted after whole module import statement
        // after importing module self.current will point to semicolon of module import statement
        let mut insert_token_at = self.current + 1; // + 1 required to insert after semicolon
        for token in imported_tokens {
            if token.kind == TokenKind::EOT { continue }
            self.tokens.insert(insert_token_at, token);
            insert_token_at += 1;
        }
    }

    fn get_tokens_from_module(&self, path: &String, prepend: Vec<char>) -> Vec<Token> {
        let module_path = Path::new(path.as_str());
        let current_module_root = Path::new(self.main_module_path.as_str()).parent().unwrap();
        let modules_relative_path_to_current_modules = current_module_root.join(module_path);
        let final_module_path = modules_relative_path_to_current_modules.as_path().to_str().unwrap();

        let mut io = io::RealIO::new();
        let src_string = io.read_src_code_from_file(final_module_path);
        match src_string {
            Ok(src) => {
                let src_chars: Vec<char> = src.chars().collect();
                let mut module_tokens = lexer::tokenize(src_chars);
                self.prepend_with_import_name(&mut module_tokens, prepend);
                return module_tokens;
            },
            Err(e) => panic!("Error opening file {}. Err: {}", final_module_path, e),
        }
    }

    fn prepend_with_import_name(&self, tokens: &mut Vec<Token>, prepend: Vec<char>) {
        for token in tokens.iter_mut() {
            if token.kind == TokenKind::Identifier {
                let mut i = 0;
                for c in prepend.iter() {
                    token.lexeme.insert(i, c.clone());
                    i += 1;
                }
                token.lexeme.insert(i, '/');
            }
        }
    }

    fn extract_filename(&self, path: &String) -> String {
        let path = Path::new(path);
        let file_name = OsStr::to_string_lossy(path.file_name().unwrap());
        file_name.to_string()
    }

    fn extract_filenames(&self, paths: &Vec<String>) -> Vec<String> {
        let mut file_names: Vec<String> = Vec::new();
        for path in paths {
            file_names.push(self.extract_filename(path));
        }
        file_names
    }

    fn extract_all_import_paths(&self, tokens: &Vec<Token>) -> Vec<String> {
        let import_stmt_start_token_indexes = self.find_all_imports_start(tokens);
        let mut modules_paths: Vec<String> = Vec::new();
        for i in import_stmt_start_token_indexes {
            modules_paths.push(self.get_module_path_from_import_stmt(tokens, i));
        }
        self.extract_filenames(&modules_paths)
    }

    fn find_all_imports_start(&self, tokens: &Vec<Token>) -> Vec<usize> {
        let mut all_imports_starting_token_index: Vec<usize> = Vec::new();
        for (i, t) in tokens.iter().enumerate() {
            if t.kind == TokenKind::Import {
                all_imports_starting_token_index.push(i)
            }
        }
        all_imports_starting_token_index
    }

    fn get_module_path_from_import_stmt(&self, tokens: &Vec<Token>,
                                        import_stmt_start_index: usize) -> String
    {
        let import_path_offset = 3;
        match tokens[import_stmt_start_index + import_path_offset].kind.clone() {
            TokenKind::String(import_path) => self.extract_filename(&import_path),
            _ => panic!("Error at line: {} import path is not valid",
                        tokens[import_stmt_start_index + import_path_offset].line),
        }
    }

    fn comment_block(&mut self) -> Stmt {
        // skipping comment block
        self.current += 1;
        // returning next statement
        return self.statements();
    }

    fn print_no_newline_stmt(&mut self) -> Stmt {
        // consuming token
        self.current += 1;
        let expr = self.expression();
        //consuming last ';' of print statement
        self.current += 1;

        Stmt::PrintNoEOL(expr)
    }

    fn re_assignment_or_func_call_stmt(&mut self) -> Stmt {
        // probably array indexing after function call won't work
        if self.tokens[self.current + 1].kind == TokenKind::ParenStart {
            // assuming its a function call statement
            let expr = self.expression();

            return Stmt::Expression(expr);
        }
        // if next token is not paren assuming it's a reassignment statement, or expression
        // statement which ony has identifier
        self.re_assignment_stmt()
    }

    fn assignment_stmt(&mut self) -> Stmt {
        // consuming var token
        self.current += 1;
        if self.tokens[self.current].kind != TokenKind::Identifier {
            panic!("Error at line: {}\nExpected a Identifier", self.tokens[self.current].line);
        }

        let var_name = self.tokens[self.current].clone();

        // consuming identifier token
        self.current += 1;
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
            // consuming '=' token
            self.current += 1;

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
                // this is identifier or indexing expression

                let mut expr = Expr::Primary(Primary::Var(self.tokens[self.current].clone()));
                // consuming identifier token
                self.current += 1;

                // this loop works for multi-dimensional or single-dimensional indexing happening, for example
                // arr[1][2] or arr[1]
                while self.tokens[self.current].kind == TokenKind::SquareBraceStart {
                    // consuming [ token
                    self.current += 1;
                    let i = self.expression();
                    if self.tokens[self.current].kind != TokenKind::SquareBraceEnd {
                        panic!("Expected ] at line {}", self.tokens[self.current].line);
                    }
                    // consuming ] token
                    self.current += 1;

                    expr = Expr::Indexing(Box::new(expr), Box::new(i));
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
                    if self.tokens[self.current].kind != TokenKind::Map {
                        panic!("Expected -> after key name at line: {}", self.tokens[self.current].line);
                    }
                    // consuming Map '->' token
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
pub fn parse(main_module_path: String, tokens: Vec<Token>) -> Vec<Stmt> {
    let mut parser = Parser::new(tokens);
    parser.main_module_path = main_module_path;
    parser.parse()
}