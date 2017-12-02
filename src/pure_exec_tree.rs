use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use super::ast;

#[derive(Clone)]
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

trait LExpr {
    fn evaluate<'a>(&self, locals: &'a mut Locals) -> &'a mut Value;
}

struct Return {
    expr: Box<Expr>,
}

impl Statement for Return {
    fn do_stmt(&self, locals: &mut Locals) -> FunctionState {
        FunctionState::Return(self.expr.evaluate(locals))
    }
}

struct Assign {
    lexpr: Box<LExpr>,
    rexpr: Box<Expr>,
}

impl Statement for Assign {
    fn do_stmt(&self, locals: &mut Locals) -> FunctionState {
        *self.lexpr.evaluate(locals) = self.rexpr.evaluate(locals);
        FunctionState::NoReturn
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

struct Identifier {
    var_id: usize,
}

impl LExpr for Identifier {
    fn evaluate<'a>(&self, locals: &'a mut Locals) -> &'a mut Value {
        &mut locals.vars[self.var_id]
    }
}

impl Expr for Identifier {
    fn evaluate(&self, locals: &Locals) -> Value {
        locals.vars[self.var_id].clone()
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

impl ScopeStack {
    fn new() -> ScopeStack {
        ScopeStack {scopes: vec![HashMap::new()], current_locals: 0, max_locals: 0}
    }

    fn declare(&mut self, name: &str) -> usize {
        let rv = self.current_locals;
        self.current_locals += 1;
        if self.current_locals > self.max_locals {
            self.max_locals = self.current_locals;
        }
        self.scopes.last_mut().unwrap().insert(name.to_string(), rv);
        println!("Allocated local {} current {} max {}", rv, self.current_locals, self.max_locals);
        rv
    }

    fn get(&self, name: &str) -> usize {
        for scope in self.scopes.iter().rev() {
            if let Some(idx) = scope.get(name) {
                return *idx
            }
        }
        panic!("Attempt to access varaible '{}' when not in scope", name)
    }
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
        println!("Building function {:?}", func);
        let (stmts, max_locals) = build_func(globals, func);
        let mut f = globals.funcs[&func.name].borrow_mut();
        f.stmts = stmts;
        f.max_locals = max_locals;
    }
}

fn build_func(globals: &Globals, func: &ast::Function) -> (Vec<Box<Statement>>, usize) {
    let mut scope_stack = ScopeStack::new();
    let stmts = build_block(globals, &mut scope_stack, &func.stmts);
    (stmts, scope_stack.max_locals)
}

fn build_block(globals: &Globals, scope_stack: &mut ScopeStack, stmts: &Vec<Box<ast::Statement>>) -> Vec<Box<Statement>> {
    let mut rv: Vec<Box<Statement>> = Vec::with_capacity(stmts.len());
    for stmt in stmts {
        match **stmt {
            ast::Statement::Return(ref expr) => {
                println!("Making Return of {:?}", expr);
                rv.push(Box::new(Return {expr: build_expr(globals, scope_stack, expr)}))
            },
            ast::Statement::Declare(ref name, ref expr) => {
                let var_id = scope_stack.declare(name);
                rv.push(Box::new(Assign {
                    lexpr: Box::new(Identifier {var_id: var_id}),
                    rexpr: build_expr(globals, scope_stack, expr),
                }))
            }
            _ => panic!("Not implemented stmt for {:?}", stmt)
        }
    };
    rv
}

fn build_expr(globals: &Globals, scope_stack: &ScopeStack, expr: &ast::Expr) -> Box<Expr> {
    use ast::Expr::*;
    use ast::BinaryOpCode::*;
    match *expr {
        Number(n) => Box::new(IntegerLiteral { value: n }),
        Identifier(ref name) => Box::new(self::Identifier {var_id: scope_stack.get(name)}),
        BinaryOp(ref l, op, ref r) => {
            let lhs = build_expr(globals, scope_stack, l);
            let rhs = build_expr(globals, scope_stack, r);
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
    println!("Making {} locals for function", func.max_locals);
    let mut locals = Locals { vars: Vec::with_capacity(func.max_locals) };
    while locals.vars.len() < func.max_locals {
        locals.vars.push(Value::Integer(0));
    }
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

    #[test]
    fn declare_and_reurn() {
        let result = compile_and_run_programme("\
            function main (args) {
                let a = 42;
                return a;
            }
        ");
        assert_eq!(result.status_code, 42);
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