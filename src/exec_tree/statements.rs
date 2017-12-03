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
        *self.lexpr.evaluate(globals, locals) = self.rexpr.evaluate(globals, locals);
        FunctionState::NoReturn
    }
}

pub fn build_block(globals: &Globals, scope_stack: &mut ScopeStack, stmts: &Vec<Box<ast::Statement>>) -> Vec<Box<Statement>> {
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
                    lexpr: Identifier::new(var_id),
                    rexpr: build_expr(globals, scope_stack, expr),
                }))
            },
            ast::Statement::Assign(ref lexpr, ref rexpr) => {
                rv.push(Box::new(Assign {
                    lexpr: build_lexpr(globals, scope_stack, lexpr),
                    rexpr: build_expr(globals, scope_stack, rexpr),
                }))
            }
            _ => panic!("Not implemented stmt for {:?}", stmt)
        }
    };
    rv
}
