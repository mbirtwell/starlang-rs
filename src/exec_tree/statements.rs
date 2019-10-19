use super::base::*;
use super::error::BuildResult;
use super::expressions::{build_expr, build_lexpr, evaluate_to_bool, Identifier};
use exec_tree::error::ExecResult;

struct Return {
    expr: ExprBox,
}

impl Statement for Return {
    fn do_stmt(&self, globals: &Globals, locals: &mut Locals) -> ExecResult<FunctionState> {
        self.expr
            .expr
            .evaluate(globals, locals)
            .map(|v| FunctionState::Return(v))
    }
}

struct Assign {
    lexpr: Box<dyn LExpr>,
    rexpr: ExprBox,
}

impl Statement for Assign {
    fn do_stmt(&self, globals: &Globals, locals: &mut Locals) -> ExecResult<FunctionState> {
        let value = self.rexpr.expr.evaluate(globals, locals)?;
        self.lexpr.assign(globals, locals, value);
        Ok(FunctionState::NoReturn)
    }
}

struct ExprStatement {
    expr: ExprBox,
}

impl Statement for ExprStatement {
    fn do_stmt(&self, globals: &Globals, locals: &mut Locals) -> ExecResult<FunctionState> {
        self.expr.expr.evaluate(globals, locals);
        Ok(FunctionState::NoReturn)
    }
}

struct IfStatement {
    expr: ExprBox,
    stmts: Vec<Box<dyn Statement>>,
}

impl Statement for IfStatement {
    fn do_stmt(&self, globals: &Globals, locals: &mut Locals) -> ExecResult<FunctionState> {
        if evaluate_to_bool(globals, locals, &self.expr) {
            exec_block(globals, locals, &self.stmts)
        } else {
            Ok(FunctionState::NoReturn)
        }
    }
}

struct WhileStatement {
    expr: ExprBox,
    stmts: Vec<Box<dyn Statement>>,
}

impl Statement for WhileStatement {
    fn do_stmt(&self, globals: &Globals, locals: &mut Locals) -> ExecResult<FunctionState> {
        while evaluate_to_bool(globals, locals, &self.expr) {
            if let FunctionState::Return(v) = exec_block(globals, locals, &self.stmts)? {
                return Ok(FunctionState::Return(v));
            }
        }
        Ok(FunctionState::NoReturn)
    }
}

pub fn build_block<'a>(
    globals: &Globals,
    scope_stack: &mut ScopeStack,
    stmts: &'a Vec<ast::Statement>,
) -> BuildResult<'a, Vec<Box<dyn Statement>>> {
    let mut rv: Vec<Box<dyn Statement>> = Vec::with_capacity(stmts.len());
    let mut failures = Vec::new();
    macro_rules! expr {
        ( $expr:expr ) => {{
            let (expr, inner_failures) = build_expr(globals, scope_stack, $expr);
            failures.extend(inner_failures);
            expr
        }};
    }
    macro_rules! stmt {
        ( $stmt:expr ) => {
            rv.push(Box::new($stmt))
        };
    }
    macro_rules! block {
        ( $stmts:expr ) => {{
            let (stmts, inner_failures) = build_block(globals, scope_stack, $stmts);
            failures.extend(inner_failures);
            stmts
        }};
    }
    for stmt in stmts {
        match *stmt {
            ast::Statement::Return(ref expr) => stmt!(Return { expr: expr!(expr) }),
            ast::Statement::Declare(ref name, ref expr) => {
                let var_id = scope_stack.declare(name);
                stmt!(Assign {
                    lexpr: Box::new(Identifier::new(var_id)),
                    rexpr: expr!(expr),
                })
            }
            ast::Statement::Assign(ref lexpr, ref rexpr) => {
                let (lexpr, inner_failures) = build_lexpr(globals, scope_stack, lexpr);
                failures.extend(inner_failures);
                stmt!(Assign {
                    lexpr: lexpr,
                    rexpr: expr!(rexpr),
                })
            }
            ast::Statement::Expr(ref expr) => stmt!(ExprStatement { expr: expr!(expr) }),
            ast::Statement::If(ref expr, ref stmts) => stmt!(IfStatement {
                expr: expr!(expr),
                stmts: block!(stmts)
            }),
            ast::Statement::While(ref expr, ref stmts) => stmt!(WhileStatement {
                expr: expr!(expr),
                stmts: block!(stmts)
            }),
        }
    }
    (rv, failures)
}
