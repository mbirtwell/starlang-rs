use super::base::*;

fn evaluate_expr_list(globals: &Globals, locals: &Locals, exprs: &[Box<Expr>]) -> Vec<Value> {
    exprs.iter().map(|ref expr| expr.evaluate(globals, locals)).collect()
}

fn evaluate_to_int(globals: &Globals, locals: &Locals, expr: &Expr) -> i32 {
    match expr.evaluate(globals, locals) {
        Value::Integer(n) => n,
        Value::Array(_) => panic!("Required int got array"),
    }
}

pub fn evaluate_to_bool(globals: &Globals, locals: &Locals, expr: &Expr) -> bool {
    match expr.evaluate(globals, locals) {
        Value::Integer(n) => n != 0,
        Value::Array(_) => unimplemented!(),
    }
}

macro_rules! evaluate_to_array {
    ($globals:expr, $locals:expr, $expr:expr, $ident:ident => $block:block) => {
        match $expr.evaluate($globals, $locals) {
            Value::Integer(_) => panic!("Required array got int"),
            Value::Array(ref $ident) => $block,
        }
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

struct StringLiteral {
    s: String,
}

impl Expr for StringLiteral {
    fn evaluate(&self, _globals: &Globals, _locals: &Locals) -> Value {
        Value::from(self.s.chars().map(|c| {Value::Integer(c as i32)}).collect::<Vec<_>>())
    }
}

struct ArrayLiteral {
    value_exprs: Vec<Box<Expr>>
}

impl Expr for ArrayLiteral {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> Value {
        Value::from(evaluate_expr_list(globals, locals, &self.value_exprs))
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
    fn assign(&self, _globals: &Globals, locals: &mut Locals, value: Value) {
        locals.vars[self.var_id] = value;
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

struct BinaryBoolOp<FnT: Fn(bool) -> bool> {
    lhs_expr: Box<Expr>,
    rhs_expr: Box<Expr>,
    should_return: FnT,
}

impl<FnT: Fn(bool) -> bool> BinaryBoolOp<FnT> {
    fn helper(&self, globals: &Globals, locals: &Locals) -> bool {
        let l = evaluate_to_bool(globals, locals, &*self.lhs_expr);
        if (self.should_return)(l) {
            l
        } else {
            evaluate_to_bool(globals, locals, &*self.rhs_expr)
        }
    }
}

impl<FnT: Fn(bool) -> bool> Expr for BinaryBoolOp<FnT> {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> Value {
        Value::Integer(if self.helper(globals, locals) {1} else {0})
    }

}

struct Call {
    func: FunctionId,
    argument_exprs: Vec<Box<Expr>>,
}

impl Expr for Call {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> Value {
        globals.lookup_func(self.func).call(
            globals, evaluate_expr_list(globals, locals, &self.argument_exprs),
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
        evaluate_to_array!(globals, locals, self.array_expr, array => {
            let array_borrow = array.borrow();
            array_borrow[index as usize].clone()
        })
    }
}

impl LExpr for Subscription {
    fn assign(&self, globals: &Globals, locals: &mut Locals, value: Value) {
        let index = evaluate_to_int(globals, locals, &*self.index_expr);
        evaluate_to_array!(globals, locals, self.array_expr, array => {
            let mut array_borrow = array.borrow_mut();
            array_borrow[index as usize] = value;
        })
    }
}

struct BoolNot {
    expr: Box<Expr>,
}

impl Expr for BoolNot {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> Value {
        Value::Integer(if evaluate_to_bool(globals, locals, &*self.expr) {0} else {1})
    }
}

struct UnaryIntegerOp<FnT: Fn(i32) -> i32> {
    expr: Box<Expr>,
    func: FnT,
}

impl<FnT: Fn(i32) -> i32> Expr for UnaryIntegerOp<FnT> {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> Value {
        Value::Integer((self.func)(evaluate_to_int(globals, locals, &*self.expr)))
    }
}

fn build_expr_list(globals: &Globals, scope_stack: &ScopeStack, exprs: &[ast::Expr]) -> Vec<Box<Expr>> {
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
        Char(c) => Box::new(IntegerLiteral {value: c as i32}),
        String(ref s) => Box::new(StringLiteral { s: s.clone() }),
        Identifier(ref name) => self::Identifier::new(scope_stack.get(name)),
        BinaryOp(ref l, op, ref r) => {
            let lhs = expr!(l);
            let rhs = expr!(r);
            macro_rules! int_op {
                ( $op:tt ) => {
                    BinaryIntegerOp::new(lhs, rhs, |l, r| {l $op r})
                }
            }
            macro_rules! cmp_op {
                ( $op:tt ) => {
                    BinaryIntegerOp::new(lhs, rhs, |l, r| { if l $op r {1} else {0} })
                };
            }
            macro_rules! bool_op {
                ( $op:expr ) => {
                    Box::new(BinaryBoolOp {lhs_expr: lhs, rhs_expr: rhs, should_return: $op})
                };
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
                LessThan => cmp_op!(<),
                MoreThan => cmp_op!(>),
                LessThanOrEqual => cmp_op!(<=),
                MoreThanOrEqual => cmp_op!(>=),
                Equal => cmp_op!(==),
                NotEqual => cmp_op!(!=),
                BoolOr => bool_op!(|v| {v}),
                BoolAnd => bool_op!(|v| {! v}),
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
        },
        UnaryOp(op, ref ast_expr) => {
            use ast::UnaryOpCode::*;
            let expr = expr!(ast_expr);
            macro_rules! int_op {
                ( $op:tt ) => {
                    Box::new(UnaryIntegerOp { expr: expr, func: |v| {$op v} })
                }
            }
            match op {
                BoolNot => Box::new(self::BoolNot {expr:expr}),
                BitNot => int_op!(!),
                Neg => int_op!(-),
                Plus => unimplemented!(),
            }
        },
        Error => panic!("This really ought not have got this far"),
    }
}

pub fn build_lexpr(globals: &Globals, scope_stack: &ScopeStack, expr: &ast::Expr) -> Box<LExpr> {
    use ast::Expr::*;
    macro_rules! expr{
        ( $expr:expr ) => {
            build_expr(globals, scope_stack, $expr)
        }
    }
    match *expr {
        Identifier(ref name) => self::Identifier::new(scope_stack.get(name)),
        Subscription(ref array_expr, ref index_expr) => {
            Box::new(self::Subscription {
                array_expr: expr!(array_expr),
                index_expr: expr!(index_expr),
            })
        }
        _ => panic!{"Not implemented or invalid l-expr for {:?} yet", expr}
    }
}
