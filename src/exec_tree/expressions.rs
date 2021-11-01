use super::base::*;
use super::error::*;

fn evaluate_expr_list(
    globals: &Globals,
    locals: &Locals,
    exprs: &[ExprBox],
) -> ExecResult<Vec<Value>> {
    exprs
        .iter()
        .map(|expr| expr.evaluate(globals, locals))
        .collect()
}

fn evaluate_to_int(globals: &Globals, locals: &Locals, expr: &ExprBox) -> ExecResult<i32> {
    match expr.evaluate(globals, locals)? {
        Value::Integer(n) => Ok(n),
        Value::Array(_) => Err(runtime_failure(
            RuntimeFailureKind::ExpectedIntGotArray,
            expr,
        )),
    }
}

pub fn evaluate_to_bool(globals: &Globals, locals: &Locals, expr: &ExprBox) -> ExecResult<bool> {
    match expr.evaluate(globals, locals)? {
        Value::Integer(n) => Ok(n != 0),
        Value::Array(_) => unimplemented!(),
    }
}

macro_rules! evaluate_to_array {
    ($globals:expr, $locals:expr, $expr:expr, $ident:ident => $block:block) => {
        match $expr.expr.evaluate($globals, $locals)? {
            Value::Integer(_) => panic!("Required array got int"),
            Value::Array(ref $ident) => $block,
        }
    };
}

struct BadExpr {}

impl Expr for BadExpr {
    fn evaluate(&self, _globals: &Globals, _locals: &Locals) -> ExecResult<Value> {
        unreachable!("Attempt to evaluate bad expression")
    }
}

struct IntegerLiteral {
    value: i32,
}

impl Expr for IntegerLiteral {
    fn evaluate(&self, _globals: &Globals, _locals: &Locals) -> ExecResult<Value> {
        Ok(Value::Integer(self.value))
    }
}

struct StringLiteral {
    s: String,
}

impl Expr for StringLiteral {
    fn evaluate(&self, _globals: &Globals, _locals: &Locals) -> ExecResult<Value> {
        Ok(Value::from(
            self.s
                .chars()
                .map(|c| Value::Integer(c as i32))
                .collect::<Vec<_>>(),
        ))
    }
}

struct ArrayLiteral {
    value_exprs: Vec<ExprBox>,
}

impl Expr for ArrayLiteral {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> ExecResult<Value> {
        Ok(Value::from(evaluate_expr_list(
            globals,
            locals,
            &self.value_exprs,
        )?))
    }
}

pub struct Identifier {
    var_id: usize,
}

impl Identifier {
    pub fn new(var_id: usize) -> Self {
        Identifier { var_id }
    }
}

impl LExpr for Identifier {
    fn assign(&self, _globals: &Globals, locals: &mut Locals, value: Value) -> ExecResult<()> {
        locals.vars[self.var_id] = value;
        Ok(())
    }
}

impl Expr for Identifier {
    fn evaluate(&self, _globals: &Globals, locals: &Locals) -> ExecResult<Value> {
        Ok(locals.vars[self.var_id].clone())
    }
}

struct BinaryIntegerOp<FnT: Fn(i32, i32) -> i32> {
    lhs_expr: ExprBox,
    rhs_expr: ExprBox,
    func: FnT,
}

impl<FnT: Fn(i32, i32) -> i32 + 'static> BinaryIntegerOp<FnT> {
    fn new(lhs_expr: ExprBox, rhs_expr: ExprBox, func: FnT) -> Self {
        BinaryIntegerOp {
            lhs_expr,
            rhs_expr,
            func,
        }
    }
}

impl<FnT: Fn(i32, i32) -> i32> Expr for BinaryIntegerOp<FnT> {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> ExecResult<Value> {
        Ok(Value::Integer((self.func)(
            evaluate_to_int(globals, locals, &self.lhs_expr)?,
            evaluate_to_int(globals, locals, &self.rhs_expr)?,
        )))
    }
}

struct BinaryBoolOp<FnT: Fn(bool) -> bool> {
    lhs_expr: ExprBox,
    rhs_expr: ExprBox,
    should_return: FnT,
}

impl<FnT: Fn(bool) -> bool> BinaryBoolOp<FnT> {
    fn helper(&self, globals: &Globals, locals: &Locals) -> ExecResult<bool> {
        let l = evaluate_to_bool(globals, locals, &self.lhs_expr)?;
        if (self.should_return)(l) {
            Ok(l)
        } else {
            evaluate_to_bool(globals, locals, &self.rhs_expr)
        }
    }
}

impl<FnT: Fn(bool) -> bool> Expr for BinaryBoolOp<FnT> {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> ExecResult<Value> {
        self.helper(globals, locals)
            .map(|v| Value::Integer(if v { 1 } else { 0 }))
    }
}

struct Call {
    func: FunctionId,
    argument_exprs: Vec<ExprBox>,
}

impl Expr for Call {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> ExecResult<Value> {
        globals.lookup_func(self.func).call(
            globals,
            evaluate_expr_list(globals, locals, &self.argument_exprs)?,
        )
    }

    fn evaluate_ex(
        &self,
        globals: &Globals,
        locals: &Locals,
        site: &CodeSite,
    ) -> ExecResult<Value> {
        self.evaluate(globals, locals).map_err(|mut e| {
            if let ExecError::RuntimeFailure(_, stack) = &mut e {
                stack.push(*site)
            }
            e
        })
    }
}

struct Subscription {
    array_expr: ExprBox,
    index_expr: ExprBox,
}

impl Expr for Subscription {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> ExecResult<Value> {
        let index = evaluate_to_int(globals, locals, &self.index_expr)?;
        evaluate_to_array!(globals, locals, self.array_expr, array => {
            let array_borrow = array.borrow();
            Ok(array_borrow[index as usize].clone())
        })
    }
}

impl LExpr for Subscription {
    fn assign(&self, globals: &Globals, locals: &mut Locals, value: Value) -> ExecResult<()> {
        let index = evaluate_to_int(globals, locals, &self.index_expr)?;
        evaluate_to_array!(globals, locals, self.array_expr, array => {
            let mut array_borrow = array.borrow_mut();
            array_borrow[index as usize] = value;
        });
        Ok(())
    }
}

struct BoolNot {
    expr: ExprBox,
}

impl Expr for BoolNot {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> ExecResult<Value> {
        Ok(Value::Integer(
            if evaluate_to_bool(globals, locals, &self.expr)? {
                0
            } else {
                1
            },
        ))
    }
}

struct UnaryIntegerOp<FnT: Fn(i32) -> i32> {
    expr: ExprBox,
    func: FnT,
}

impl<FnT: Fn(i32) -> i32> Expr for UnaryIntegerOp<FnT> {
    fn evaluate(&self, globals: &Globals, locals: &Locals) -> ExecResult<Value> {
        Ok(Value::Integer((self.func)(evaluate_to_int(
            globals, locals, &self.expr,
        )?)))
    }
}

fn build_expr_list<'a>(
    globals: &Globals,
    scope_stack: &ScopeStack,
    exprs: &'a [ast::Expr],
) -> BuildResult<'a, Vec<ExprBox>> {
    let mut rv = Vec::new();
    let mut failures = StaticAnalysisErrors::new();
    for expr in exprs {
        let (e, f) = build_expr(globals, scope_stack, expr);
        rv.push(e);
        failures.extend(f);
    }
    (rv, failures)
}

pub fn build_expr<'a>(
    globals: &Globals,
    scope_stack: &ScopeStack,
    expr: &'a ast::Expr,
) -> BuildResult<'a, ExprBox> {
    use ast::BinaryOpCode::*;
    use ast::ExprKind::*;
    let mut failures = StaticAnalysisErrors::new();
    macro_rules! result {
        ( $expr:expr ) => {
            (
                ExprBox {
                    expr: Box::new($expr),
                    site: CodeSite {
                        start: expr.start,
                        end: expr.end,
                    },
                },
                failures,
            )
        };
    }
    macro_rules! failure {
        ( $failure:expr ) => {{
            failures.push($failure);
            result!(BadExpr {})
        }};
    }
    macro_rules! expr {
        ( $expr:expr ) => {{
            let (ex, inner_failures) = build_expr(globals, scope_stack, $expr);
            failures.extend(inner_failures);
            ex
        }};
    }
    macro_rules! expr_list {
        ( $expr:expr ) => {{
            let (rv, inner_failures) = build_expr_list(globals, scope_stack, $expr);
            failures.extend(inner_failures);
            rv
        }};
    }
    match expr.kind {
        Number(n) => result!(IntegerLiteral { value: n }),
        Char(c) => result!(IntegerLiteral { value: c as i32 }),
        String(s) => result!(StringLiteral { s: s.to_string() }),
        Identifier(name) => result!(self::Identifier::new(scope_stack.get(name))),
        BinaryOp(ref l, op, ref r) => {
            let lhs = expr!(l);
            let rhs = expr!(r);
            macro_rules! int_op {
                ( $op:tt ) => {
                    result!(BinaryIntegerOp::new(lhs, rhs, |l, r| {l $op r}))
                }
            }
            macro_rules! cmp_op {
                ( $op:tt ) => {
                    result!(BinaryIntegerOp::new(lhs, rhs, |l, r| { if l $op r {1} else {0} }))
                };
            }
            macro_rules! bool_op {
                ( $op:expr ) => {
                    result!(BinaryBoolOp {
                        lhs_expr: lhs,
                        rhs_expr: rhs,
                        should_return: $op
                    })
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
                BoolOr => bool_op!(|v| { v }),
                BoolAnd => bool_op!(|v| { !v }),
            }
        }
        Call(ref fname, ref argument_exprs) => {
            let argument_exprs = expr_list!(argument_exprs);
            if let Some(func) = globals.reference_func(fname) {
                result!(self::Call {
                    func,
                    argument_exprs,
                })
            } else {
                failure!(StaticAnalysisError::CallUnknownFunction(
                    fname.to_string(),
                    expr.start,
                    expr.end,
                ))
            }
        }
        Array(ref value_exprs) => result!(ArrayLiteral {
            value_exprs: expr_list!(value_exprs)
        }),
        Subscription(ref array_expr, ref index_expr) => result!(self::Subscription {
            array_expr: expr!(array_expr),
            index_expr: expr!(index_expr),
        }),
        UnaryOp(op, ref ast_expr) => {
            use ast::UnaryOpCode::*;
            let expr = expr!(ast_expr);
            macro_rules! int_op {
                ( $op:tt ) => {
                    result!(UnaryIntegerOp { expr, func: |v| {$op v} })
                }
            }
            match op {
                BoolNot => result!(self::BoolNot { expr }),
                BitNot => int_op!(!),
                Neg => int_op!(-),
                Plus => unimplemented!(),
            }
        }
        Error => panic!("This really ought not have got this far"),
    }
}

pub fn build_lexpr<'a>(
    globals: &Globals,
    scope_stack: &ScopeStack,
    expr: &'a ast::Expr,
) -> BuildResult<'a, Box<dyn LExpr>> {
    use ast::ExprKind::*;
    let mut failures = StaticAnalysisErrors::new();
    macro_rules! result {
        ( $expr:expr ) => {
            (Box::new($expr), failures)
        };
    }
    macro_rules! expr {
        ( $expr:expr ) => {{
            let (ex, inner_failures) = build_expr(globals, scope_stack, $expr);
            failures.extend(inner_failures);
            ex
        }};
    }
    match &expr.kind {
        Identifier(name) => result!(self::Identifier::new(scope_stack.get(name))),
        Subscription(array_expr, ref index_expr) => result!(self::Subscription {
            array_expr: expr!(array_expr),
            index_expr: expr!(index_expr),
        }),
        _ => panic! {"Not implemented or invalid l-expr for {:?} yet", expr},
    }
}
