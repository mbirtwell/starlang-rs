use super::base::*;
use std::rc::Rc;
use std::cell::RefCell;
use super::main::{exec_func};

fn evaluate_expr_list(globals: &Globals, locals: &Locals, exprs: &[Box<Expr>]) -> Vec<Value> {
    exprs.iter().map(|ref expr| expr.evaluate(globals, locals)).collect()
}

fn evaluate_to_int(globals: &Globals, locals: &Locals, expr: &Expr) -> i32 {
    match expr.evaluate(globals, locals) {
        Value::Integer(n) => n,
        Value::Array(_) => panic!("Required int got array"),
    }
}

fn evaluate_to_array(globals: &Globals, locals: &Locals, expr: &Expr) -> Array {
    match expr.evaluate(globals, locals) {
        Value::Integer(_) => panic!("Required array got int"),
        Value::Array(ref array) => array.clone(),
    }
}

struct IntegerLiteral {
    value: i32,
}

impl Expr for IntegerLiteral {
    fn evaluate(&self, _globals: &Globals, _locals: &Locals) -> Value {
        Value::Integer(self.value)
    }
}

struct ArrayLiteral {
    value_exprs: Vec<Box<Expr>>
}

impl Expr for ArrayLiteral {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> Value {
        Value::Array(Rc::new(RefCell::new(
            evaluate_expr_list(globals, locals, &self.value_exprs).into_boxed_slice()
        )))
    }
}

pub struct Identifier {
    var_id: usize,
}

impl Identifier {
    pub fn new(var_id: usize) -> Box<Identifier> {
        Box::new(Identifier {var_id: var_id})
    }
}

impl LExpr for Identifier {
    fn evaluate<'a>(&self, _globals: &Globals, locals: &'a mut Locals) -> &'a mut Value {
        &mut locals.vars[self.var_id]
    }
}

impl Expr for Identifier {
    fn evaluate(&self, _globals: &Globals, locals: &Locals) -> Value {
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

impl<FnT: Fn(i32, i32) -> i32> Expr for BinaryIntegerOp<FnT> {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> Value {
        Value::Integer((self.func)(
            evaluate_to_int(globals, locals, &*self.lhs_expr),
            evaluate_to_int(globals, locals, &*self.rhs_expr),
        ))
    }
}

struct Call {
    func: FunctionId,
    argument_exprs: Vec<Box<Expr>>,
}

impl Expr for Call {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> Value {
        exec_func(
            globals,
            globals.lookup_func(self.func),
            evaluate_expr_list(globals, locals, &self.argument_exprs),
        )
    }
}

struct Subscription {
    array_expr: Box<Expr>,
    index_expr: Box<Expr>,
}

impl Expr for Subscription {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> Value {
        let index = evaluate_to_int(globals, locals, &*self.index_expr);
        let array = evaluate_to_array(globals, locals, &*self.array_expr);
        {
            let array_borrow = array.borrow();
            array_borrow[index as usize].clone()
        }
    }
}

fn build_expr_list(globals: &Globals, scope_stack: &ScopeStack, exprs: &[Box<ast::Expr>]) -> Vec<Box<Expr>> {
    exprs.iter().map(
        |ref expr| build_expr(globals, scope_stack, expr)
    ).collect()
}

pub fn build_expr(globals: &Globals, scope_stack: &ScopeStack, expr: &ast::Expr) -> Box<Expr> {
    use ast::Expr::*;
    use ast::BinaryOpCode::*;
    macro_rules! expr{
        ( $expr:expr ) => {
            build_expr(globals, scope_stack, $expr)
        }
    }
    macro_rules! expr_list{
        ( $expr:expr ) => {
            build_expr_list(globals, scope_stack, $expr)
        }
    }
    match *expr {
        Number(n) => Box::new(IntegerLiteral { value: n }),
        Identifier(ref name) => self::Identifier::new(scope_stack.get(name)),
        BinaryOp(ref l, op, ref r) => {
            let lhs = expr!(l);
            let rhs = expr!(r);
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
        },
        Call(ref fname, ref argument_exprs) => {
            Box::new(self::Call {
                func: globals.reference_func(fname),
                argument_exprs: expr_list!(argument_exprs),
            })
        },
        Array(ref value_exprs) => {
            Box::new(ArrayLiteral {
                value_exprs: expr_list!(value_exprs)
            })
        },
        Subscription(ref array_expr, ref index_expr) => {
            Box::new(self::Subscription {
                array_expr: expr!(array_expr),
                index_expr: expr!(index_expr),
            })
        }
        _ => panic!{"Not implemented expr for {:?} yet", expr}
    }
}

pub fn build_lexpr(globals: &Globals, scope_stack: &ScopeStack, expr: &ast::Expr) -> Box<LExpr> {
    use ast::Expr::*;
    match *expr {
        Identifier(ref name) => self::Identifier::new(scope_stack.get(name)),
        _ => panic!{"Not implemented or invalid l-expr for {:?} yet", expr}
    }
}
