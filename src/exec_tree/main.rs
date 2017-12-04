use super::base::*;
use super::statements::build_block;

fn collect_funcs(globals: &mut Globals, programme: &Vec<ast::Function>) {
    for func in programme {
        globals.declare_func(func);
    }
}

fn build_funcs(globals: &mut Globals, programme: &Vec<ast::Function>) {
    for func in programme {
        println!("Building function {:?}", func);
        let (stmts, max_locals) = build_func(globals, func);
        globals.define_func(&func.name, stmts, max_locals);
    }
}

fn build_func(globals: &Globals, func: &ast::Function) -> (Vec<Box<Statement>>, usize) {
    let mut scope_stack = ScopeStack::new();
    for arg in &func.arguments {
        scope_stack.declare(arg);
    }
    let stmts = build_block(globals, &mut scope_stack, &func.stmts);
    (stmts, scope_stack.get_max_locals())
}

pub fn exec(programme: &Vec<ast::Function>) -> i32 {
    let mut globals = Globals::new();
    collect_funcs(&mut globals, programme);
    if !globals.has_main() {
        panic!("No main function defined");
    }
    build_funcs(&mut globals, programme);
    {
        let main_func = globals.get_main();
        match exec_func(&globals, &main_func, vec![Value::Integer(0)]) {
            Value::Integer(status_code) => status_code,
            Value::Array(_) => panic!("Array returned from main. Requires int.")
        }
    }
}

pub fn exec_func(globals: &Globals, func: &Function, args: Vec<Value>) -> Value {
    println!("Making {} locals for function", func.max_locals);
    let mut locals = Locals { vars: args };
    locals.vars.reserve(func.max_locals);
    while locals.vars.len() < func.max_locals {
        locals.vars.push(Value::Integer(0));
    }
    match exec_block(globals, &mut locals, &func.stmts) {
        FunctionState::Return(val) => val,
        FunctionState::NoReturn => Value::Integer(0),
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

