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

pub struct Function {
    pub stmts: Vec<Box<Statement>>,
    pub max_locals: usize,
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
    functions: Vec<Function>,
}

impl Globals {
    pub fn new() -> Globals {
        Globals {
            function_declarations: HashMap::new(),
            functions: Vec::new(),
        }
    }
    pub fn declare_func(&mut self, func: &ast::Function) {
        let id = FunctionId { idx: self.function_declarations.len() };
        self.function_declarations.insert(
            func.name.clone(),
            FunctionDeclaration {
                id: id,
            },
        );
    }
    pub fn has_main(&self) -> bool { self.function_declarations.contains_key("main") }
    pub fn get_main(&self) -> &Function {
        self.lookup_func(self.reference_func("main"))
    }
    pub fn define_func(&mut self, name: &str, stmts: Vec<Box<Statement>>, max_locals: usize) {
        match self.function_declarations.get(name) {
            Some(ref decl) => {
                if self.functions.len() != decl.id.idx {
                    panic!("Attempting to define function {} out of declaration order.", name)
                }
                self.functions.push(Function {
                    stmts: stmts,
                    max_locals: max_locals,
                })
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
    pub fn lookup_func(&self, func_id: FunctionId) -> &Function {
        &self.functions[func_id.idx]
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
    fn evaluate<'a>(&self, globals: &Globals, locals: &'a mut Locals) -> &'a mut Value;
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
