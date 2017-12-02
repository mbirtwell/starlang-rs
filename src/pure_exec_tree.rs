use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use super::ast;

enum Value {
    Integer(i32),
    //Array(Rc<RefCell<[Value]>>),
}

struct Function {
    name: String,
    arguments: Vec<String>,
    stmts: Vec<Box<Statement>>,
    max_locals: usize,
}

struct Globals {
    funcs: HashMap<String, Rc<RefCell<Function>>>,
}

struct Locals {
    vars: Vec<Value>,
}

enum FunctionState {
    Return(Value),
    NoReturn,
}

trait Statement {
    fn do_stmt(&self, locals: &mut Locals) -> FunctionState;
}

trait Expr {
    fn evaluate(&self, locals: &Locals) -> Value;
}

struct Return {
    expr: Box<Expr>,
}

impl Statement for Return {
    fn do_stmt(&self, locals: &mut Locals) -> FunctionState {
        FunctionState::Return(self.expr.evaluate(locals))
    }
}

struct IntegerLiteral {
    value: i32,
}

impl Expr for IntegerLiteral {
    fn evaluate(&self, locals: &Locals) -> Value {
        Value::Integer(self.value)
    }
}

struct BinaryIntegerOp<FnT: Fn(i32, i32) -> i32>  {
    lhs_expr: Box<Expr>,
    rhs_expr: Box<Expr>,
    func: FnT,
}

impl<FnT: Fn(i32, i32) -> i32 + 'static> BinaryIntegerOp<FnT> {
    fn new(lhs_expr: Box<Expr>, rhs_expr: Box<Expr>, func: FnT) -> Box<Expr> {
        Box::new(BinaryIntegerOp {lhs_expr: lhs_expr, rhs_expr: rhs_expr, func: func})
    }
}

fn evaluate_to_int(locals: &Locals, expr: &Expr) -> i32 {
    match expr.evaluate(locals) {
        Value::Integer(n) => n,
    }
}

impl<FnT: Fn(i32, i32) -> i32> Expr for BinaryIntegerOp<FnT> {
    fn evaluate(&self, locals: &Locals) -> Value {
        Value::Integer((self.func)(
            evaluate_to_int(locals, &*self.lhs_expr),
            evaluate_to_int(locals, &*self.rhs_expr),
        ))
    }
}

struct ScopeStack {
    scopes: Vec<HashMap<String, usize>>,
    current_locals: usize,
    max_locals: usize,
}

fn collect_funcs(globals: &mut Globals, programme: &Vec<ast::Function>) {
    for func in programme {
        globals.funcs.insert(
            func.name.clone(),
            Rc::new(RefCell::new(Function {
                name: func.name.clone(),
                arguments: func.arguments.clone(),
                stmts: Vec::new(),
                max_locals: 0,
            })),
        );
    }
}

fn build_funcs(globals: &mut Globals, programme: &Vec<ast::Function>) {
    for func in programme {
        print!("Building function {:?}", func);
        let stmts = build_func(globals, func);
        globals.funcs[&func.name].borrow_mut().stmts = stmts;
    }
}

fn build_func(globals: &Globals, func: &ast::Function) -> Vec<Box<Statement>> {
    let mut scope_stack = ScopeStack {scopes: Vec::new(), current_locals: 0, max_locals: 0};
    build_block(globals, &mut scope_stack, &func.stmts)
}

fn build_block(globals: &Globals, scope_stack: &mut ScopeStack, stmts: &Vec<Box<ast::Statement>>) -> Vec<Box<Statement>> {
    let mut rv: Vec<Box<Statement>> = Vec::with_capacity(stmts.len());
    for stmt in stmts {
        match **stmt {
            ast::Statement::Return(ref expr) => {
                print!("Making Return of {:?}", expr);
                rv.push(Box::new(Return {expr: build_expr(globals, expr)}))
            },
            _ => panic!("Not implemented stmt for {:?}", stmt)
        }
    };
    rv
}

fn build_expr(globals: &Globals, expr: &ast::Expr) -> Box<Expr> {
    use ast::Expr::*;
    use ast::BinaryOpCode::*;
    match *expr {
        Number(n) => Box::new(IntegerLiteral { value: n }),
        BinaryOp(ref l, op, ref r) => {
            let lhs = build_expr(globals, l);
            let rhs = build_expr(globals, r);
            macro_rules! int_op{
                ( $op:tt ) => {
                    BinaryIntegerOp::new(lhs, rhs, |l, r| {l $op r})
                }
            }
            match op {
                Add => int_op!(+),
                Sub => int_op!(-),
                Mul => int_op!(*),
                Div => int_op!(/),
                Mod => int_op!(%),
                LeftShift => int_op!(<<),
                RightShift => int_op!(>>),
                BitOr => int_op!(|),
                BitAnd => int_op!(&),
                BitXor => int_op!(^),
                _ => panic!("Not implemented op code for {:?}", op),
            }
        }
        _ => panic!{"Not implemented expr for {:?} yet", expr}
    }
}

pub fn exec(programme: &Vec<ast::Function>) -> i32 {
    let mut globals = Globals { funcs: HashMap::new() };
    collect_funcs(&mut globals, programme);
    if !globals.funcs.contains_key("main") {
        panic!("No main function defined");
    }
    build_funcs(&mut globals, programme);
    let main_value = match globals.funcs.get("main") {
        Some(ref func) => exec_func(&func.borrow()),
        None => unreachable!(),
    };
    match main_value {
        Value::Integer(status_code) => status_code,
    }
}

fn exec_func(func: &Function) -> Value {
    let mut locals = Locals { vars: Vec::with_capacity(func.max_locals) };
    match exec_block(&mut locals, &func.stmts) {
        FunctionState::Return(val) => val,
        FunctionState::NoReturn => Value::Integer(0),
    }
}

fn exec_block(locals: &mut Locals, stmts: &[Box<Statement>]) -> FunctionState {
    for stmt in stmts {
        match stmt.do_stmt(locals) {
            FunctionState::Return(val) => return FunctionState::Return(val),
            FunctionState::NoReturn => {},
        }
    };
    FunctionState::NoReturn
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::grammar::{parse_Programme};

    struct ProgResult {
        status_code: i32,
    }

    fn compile_and_run_programme(text: &str) -> ProgResult {
        let prog = parse_Programme(text).unwrap();
        ProgResult { status_code: exec(&prog) }
    }

    #[test]
    fn noop_programme() {
        let result = compile_and_run_programme("\
            function main (args) {
            }
        ");
        assert_eq!(result.status_code, 0);
    }

    #[test]
    fn main_return_status_code() {
        let result = compile_and_run_programme("\
            function main (args) {
                return 3;
            }
        ");
        assert_eq!(result.status_code, 3);
    }

    #[test]
    fn return_expression() {
        let result = compile_and_run_programme("\
            function main (args) {
                return 2 + 3;
            }
        ");
        assert_eq!(result.status_code, 5);
    }

    #[test]
    fn return_more_maths() {
        let result = compile_and_run_programme("\
            function main (args) {
                return (2 * 5 - 1) % 5 ;
            }
        ");
        assert_eq!(result.status_code, 4);
    }

    #[test]
    fn return_division() {
        let result = compile_and_run_programme("\
            function main (args) {
                return 5 / 2;
            }
        ");
        assert_eq!(result.status_code, 2);
    }

    #[test]
    fn return_bit_manipulation() {
        let result = compile_and_run_programme("\
            function main (args) {
                return 1 << 2 | 64 >> 3 | 255 & 64 | 255 - 32 ^ 255;
            }
        ");
        assert_eq!(result.status_code, 0x6c);
    }

//    #[test]
//    fn comparisions() {
//        let result = compile_and_run_programme("\
//            function main (args) {
//                return
//                    (1 < 3) << 0
//                    (4 < 3) << 1
//                    (4 > 3) << 2
//                    (4 > 6) << 3
//                    (4 > 6) << 3
//                ;
//            }
//        ");
//        assert_eq!(result.status_code, 0x6c);
//    }

}