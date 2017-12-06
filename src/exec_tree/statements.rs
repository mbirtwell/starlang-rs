use super::base::*;
use super::expressions::{build_expr, build_lexpr, Identifier};

struct Return {
    expr: Box<Expr>,
}

impl Statement for Return {
    fn do_stmt(&self, globals: &Globals, locals: &mut Locals) -> FunctionState {
        FunctionState::Return(self.expr.evaluate(globals, locals))
    }
}

struct Assign {
    lexpr: Box<LExpr>,
    rexpr: Box<Expr>,
}

impl Statement for Assign {
    fn do_stmt(&self, globals: &Globals, locals: &mut Locals) -> FunctionState {
        let value = self.rexpr.evaluate(globals, locals);
        self.lexpr.assign(globals, locals, value);
        FunctionState::NoReturn
    }
}

struct ExprStatement {
    expr: Box<Expr>,
}

impl Statement for ExprStatement {
    fn do_stmt(&self, globals: &Globals, locals: &mut Locals) -> FunctionState {
        self.expr.evaluate(globals, locals);
        FunctionState::NoReturn
    }

}

pub fn build_block(globals: &Globals, scope_stack: &mut ScopeStack, stmts: &Vec<Box<ast::Statement>>) -> Vec<Box<Statement>> {
    let mut rv: Vec<Box<Statement>> = Vec::with_capacity(stmts.len());
    macro_rules! expr{
        ( $expr:expr ) => {
            build_expr(globals, scope_stack, $expr)
        }
    }
    macro_rules! stmt {
        ( $stmt:expr ) => {
            rv.push(Box::new($stmt))
        };
    }
    for stmt in stmts {
        match **stmt {
            ast::Statement::Return(ref expr) => {
                stmt!(Return {expr: expr!(expr)})
            },
            ast::Statement::Declare(ref name, ref expr) => {
                let var_id = scope_stack.declare(name);
                stmt!(Assign {
                    lexpr: Identifier::new(var_id),
                    rexpr: expr!(expr),
                })
            },
            ast::Statement::Assign(ref lexpr, ref rexpr) => {
                stmt!(Assign {
                    lexpr: build_lexpr(globals, scope_stack, lexpr),
                    rexpr: expr!(rexpr),
                })
            },
            ast::Statement::Expr(ref expr) => {
                stmt!(ExprStatement {expr: expr!(expr)})
            }
            _ => panic!("Not implemented stmt for {:?}", stmt)
        }
    };
    rv
}
