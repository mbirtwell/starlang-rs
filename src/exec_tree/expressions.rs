use super::base::*;
use super::main::{exec_func};

struct IntegerLiteral {
    value: i32,
}

impl Expr for IntegerLiteral {
    fn evaluate(&self, _globals: &Globals, _locals: &Locals) -> Value {
        Value::Integer(self.value)
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

fn evaluate_to_int(globals: &Globals, locals: &Locals, expr: &Expr) -> i32 {
    match expr.evaluate(globals, locals) {
        Value::Integer(n) => n,
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
            self.argument_exprs.iter().map(|ref expr| expr.evaluate(globals, locals)).collect(),
        )
    }
}

pub fn build_expr(globals: &Globals, scope_stack: &ScopeStack, expr: &ast::Expr) -> Box<Expr> {
    use ast::Expr::*;
    use ast::BinaryOpCode::*;
    match *expr {
        Number(n) => Box::new(IntegerLiteral { value: n }),
        Identifier(ref name) => self::Identifier::new(scope_stack.get(name)),
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
        },
        Call(ref fname, ref argument_exprs) => {
            Box::new(self::Call {
                func: globals.reference_func(fname),
                argument_exprs: argument_exprs.iter().map(
                    |ref expr| build_expr(globals, scope_stack, expr)
                ).collect(),
            })
        },
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
