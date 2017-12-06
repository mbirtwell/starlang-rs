use std::io::{Read,Write};
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

fn convert_args_to_values(args: Vec<String>) -> Value {
    Value::from(args.iter().map(|arg| {
        Value::from(arg.chars().map(|char| {
            Value::Integer(char as i32)
        }).collect::<Vec<_>>())
    }).collect::<Vec<_>>())
}

pub fn exec(programme: &Vec<ast::Function>, args: Vec<String>, input: Box<Read>, output: &mut Write) -> i32 {
    let mut globals = Globals::new(input, output);
    collect_funcs(&mut globals, programme);
    if !globals.has_main() {
        panic!("No main function defined");
    }
    build_funcs(&mut globals, programme);
    {
        let main_func = globals.get_main();
        match main_func.call(&globals, vec![convert_args_to_values(args)]) {
            Value::Integer(status_code) => status_code,
            Value::Array(_) => panic!("Array returned from main. Requires int.")
        }
    }
}

