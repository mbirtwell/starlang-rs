use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
pub use super::super::ast;

pub type Array = Rc<RefCell<Box<[Value]>>>;

#[derive(Clone)]
pub enum Value {
    Integer(i32),
    Array(Array),
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Value {
        Value::Array(Rc::new(RefCell::new(value.into_boxed_slice())))
    }
}

pub trait Callable {
    fn call(&self, globals: &Globals, args: Vec<Value>) -> Value;
}

struct StarLangFunction {
    stmts: Vec<Box<Statement>>,
    max_locals: usize,
}

struct PlatformFunction {
    func: Box<Fn(Vec<Value>) -> Value>,
}

#[derive(Copy,Clone)]
pub struct FunctionId {
    idx: usize,
}

struct FunctionDeclaration {
    id: FunctionId,
}

pub struct Globals {
    function_declarations: HashMap<String, FunctionDeclaration>,
    functions: Vec<Box<Callable>>,
}

fn starlang_new(args: Vec<Value>) -> Value {
    match args[0] {
        Value::Integer(n) => {
            Value::from(vec![Value::Integer(0); n as usize])
        },
        _ => {
            panic!("platform function 'new' expected int but recieved array");
        }
    }
}

impl Globals {
    pub fn new() -> Globals {
        let mut rv = Globals {
            function_declarations: HashMap::new(),
            functions: Vec::new(),
        };
        rv.define_platform_func("new", Box::new(starlang_new));
        rv
    }
    pub fn declare_func(&mut self, func: &ast::Function) {
        let id = self.next_func_id();
        self.function_declarations.insert(
            func.name.clone(),
            FunctionDeclaration {
                id: id,
            },
        );
    }
    pub fn has_main(&self) -> bool { self.function_declarations.contains_key("main") }
    pub fn get_main(&self) -> &Callable {
        self.lookup_func(self.reference_func("main"))
    }
    pub fn define_func(&mut self, name: &str, stmts: Vec<Box<Statement>>, max_locals: usize) {
        match self.function_declarations.get(name) {
            Some(ref decl) => {
                if self.functions.len() != decl.id.idx {
                    panic!("Attempting to define function {} out of declaration order.", name)
                }
                self.functions.push(Box::new(StarLangFunction {
                    stmts: stmts,
                    max_locals: max_locals,
                }))
            },
            None => unreachable!("Attempting to define undeclared function {}", name),
        }
    }
    pub fn reference_func(&self, name: &str) -> FunctionId {
        match self.function_declarations.get(name) {
            Some(ref decl) => decl.id,
            None => panic!("Attempting to use undeclard function {}", name),
        }
    }
    pub fn lookup_func(&self, func_id: FunctionId) -> &Callable {
        &*self.functions[func_id.idx]
    }
    fn next_func_id(&self) -> FunctionId {
        FunctionId { idx: self.function_declarations.len() }
    }
    fn define_platform_func(&mut self, name: &str, func: Box<Fn(Vec<Value>) -> Value>) {
        let id = self.next_func_id();
        self.function_declarations.insert(
            name.to_string(),
            FunctionDeclaration {
                id: id,
            },
        );
        if self.functions.len() != id.idx {
            panic!("Attempting to define function {} out of declaration order.", name)
        }
        self.functions.push(Box::new(PlatformFunction {
            func: func,
        }))
    }
}

pub struct Locals {
    pub vars: Vec<Value>,
}

pub enum FunctionState {
    Return(Value),
    NoReturn,
}

pub trait Statement {
    fn do_stmt(&self, globals: &Globals, locals: &mut Locals) -> FunctionState;
}

pub trait Expr {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> Value;
}

pub trait LExpr {
    fn assign(&self, globals: &Globals, locals: &mut Locals, value: Value);
}

pub struct ScopeStack {
    scopes: Vec<HashMap<String, usize>>,
    current_locals: usize,
    max_locals: usize,
}

impl ScopeStack {
    pub fn new() -> ScopeStack {
        ScopeStack {scopes: vec![HashMap::new()], current_locals: 0, max_locals: 0}
    }

    pub fn declare(&mut self, name: &str) -> usize {
        let rv = self.current_locals;
        self.current_locals += 1;
        if self.current_locals > self.max_locals {
            self.max_locals = self.current_locals;
        }
        self.scopes.last_mut().unwrap().insert(name.to_string(), rv);
        println!("Allocated local {} current {} max {}", rv, self.current_locals, self.max_locals);
        rv
    }

    pub fn get(&self, name: &str) -> usize {
        for scope in self.scopes.iter().rev() {
            if let Some(idx) = scope.get(name) {
                return *idx
            }
        }
        panic!("Attempt to access varaible '{}' when not in scope", name)
    }

    pub fn get_max_locals(&self) -> usize {
        self.max_locals
    }
}

impl Callable for StarLangFunction {
    fn call(&self, globals: &Globals, args: Vec<Value>) -> Value {
        let mut locals = Locals { vars: args };
        locals.vars.reserve(self.max_locals);
        while locals.vars.len() < self.max_locals {
            locals.vars.push(Value::Integer(0));
        }
        match exec_block(globals, &mut locals, &self.stmts) {
            FunctionState::Return(val) => val,
            FunctionState::NoReturn => Value::Integer(0),
        }

    }
}

fn exec_block(globals: &Globals, locals: &mut Locals, stmts: &[Box<Statement>]) -> FunctionState {
    for stmt in stmts {
        match stmt.do_stmt(globals, locals) {
            FunctionState::Return(val) => return FunctionState::Return(val),
            FunctionState::NoReturn => {},
        }
    };
    FunctionState::NoReturn
}

impl Callable for PlatformFunction {
    fn call(&self, _globals: &Globals, args: Vec<Value>) -> Value {
        (self.func)(args)
    }
}