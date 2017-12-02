use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{RefCell, Ref};
pub use super::super::ast;

#[derive(Clone)]
pub enum Value {
    Integer(i32),
    //Array(Rc<RefCell<[Value]>>),
}

pub struct Function {
    pub name: String,
    pub arguments: Vec<String>,
    pub stmts: Vec<Box<Statement>>,
    pub max_locals: usize,
}

pub struct Globals {
    pub funcs: HashMap<String, Rc<RefCell<Function>>>,
}

impl Globals {
    pub fn new() -> Globals {
        Globals { funcs: HashMap::new() }
    }
    pub fn add_func(&mut self, func: &ast::Function) {
        self.funcs.insert(
            func.name.clone(),
            Rc::new(RefCell::new(Function {
                name: func.name.clone(),
                arguments: func.arguments.clone(),
                stmts: Vec::new(),
                max_locals: 0,
            })),
        );
    }
    pub fn has_main(&self) -> bool { self.funcs.contains_key("main") }
    pub fn get_main(&self) -> Ref<Function> {
        match self.funcs.get("main") {
            Some(ref func) => func.borrow(),
            None => unreachable!(),
        }
    }
    pub fn define_func(&mut self, name: &str, stmts: Vec<Box<Statement>>, max_locals: usize) {
        let mut f = self.funcs[name].borrow_mut();
        f.stmts = stmts;
        f.max_locals = max_locals;
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
    fn do_stmt(&self, locals: &mut Locals) -> FunctionState;
}

pub trait Expr {
    fn evaluate(&self, locals: &Locals) -> Value;
}

pub trait LExpr {
    fn evaluate<'a>(&self, locals: &'a mut Locals) -> &'a mut Value;
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
