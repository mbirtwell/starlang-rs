use super::base::*;
use super::error::*;
use super::statements::build_block;
use std::io::{Read, Write};

fn collect_funcs(globals: &mut Globals, programme: &[ast::Function]) {
    for func in programme {
        globals.declare_func(func);
    }
}

fn build_funcs(globals: &mut Globals, programme: &[ast::Function]) -> ExecResult<()> {
    let mut failures = StaticAnalysisErrors::new();
    for func in programme {
        let ((stmts, max_locals), func_failures) = build_func(globals, func);
        globals.define_func(&func.name, stmts, max_locals);
        failures.extend(func_failures);
    }
    if !failures.is_empty() {
        Err(ExecError::StaticAnalysisFailed(failures))
    } else {
        Ok(())
    }
}

fn build_func<'a>(
    globals: &Globals,
    func: &'a ast::Function,
) -> BuildResult<'a, (Vec<Box<dyn Statement>>, usize)> {
    let mut scope_stack = ScopeStack::new();
    for arg in &func.arguments {
        scope_stack.declare(arg);
    }
    let (stmts, failures) = build_block(globals, &mut scope_stack, &func.stmts);
    ((stmts, scope_stack.get_max_locals()), failures)
}

fn convert_args_to_values(args: Vec<String>) -> Value {
    Value::from(
        args.iter()
            .map(|arg| {
                Value::from(
                    arg.chars()
                        .map(|char| Value::Integer(char as i32))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>(),
    )
}

pub fn exec<'a>(
    programme: &'a [ast::Function],
    args: Vec<String>,
    input: &mut dyn Read,
    output: &mut dyn Write,
) -> ExecResult<i32> {
    let mut globals = Globals::new(input, output);
    collect_funcs(&mut globals, programme);
    assert!(globals.has_main(), "No main function defined");
    build_funcs(&mut globals, programme)?;
    {
        let main_func = globals.get_main();
        match main_func.call(&globals, vec![convert_args_to_values(args)])? {
            Value::Integer(status_code) => Ok(status_code),
            Value::Array(_) => panic!("Array returned from main. Requires int."),
        }
    }
}
