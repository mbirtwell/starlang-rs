use std::fmt::{Debug, Formatter, Error};


pub struct Function {
    pub name: String,
    pub arguments: Vec<String>,
    pub stmts: Vec<Statement>,
}

impl Function {
    pub fn new(name: String, arguments: Vec<String>, stmts: Vec<Statement>) -> Function {
        return Function {name: name, arguments: arguments, stmts: stmts};
    }
}

pub enum Statement {
    Expr(Expr),
    Return(Expr),
    Assign(Expr, Expr),
    Declare(String, Expr),
    If(Expr, Vec<Statement>),
    While(Expr, Vec<Statement>),
}

pub enum Expr {
    Number(i32),
    Char(char),
    String(String),
    Array(Vec<Expr>),
    BinaryOp(Box<Expr>, BinaryOpCode, Box<Expr>),
    UnaryOp(UnaryOpCode, Box<Expr>),
    Call(String, Vec<Expr>),
    Identifier(String),
    Subscription(Box<Expr>, Box<Expr>),
    Error,
}

impl Expr {
    pub fn new_binary_op(lhs: Expr, op: BinaryOpCode, rhs: Expr) -> Expr {
        Expr::BinaryOp(Box::new(lhs), op, Box::new(rhs))
    }
    pub fn new_unary_op(op: UnaryOpCode, expr: Expr) -> Expr {
        Expr::UnaryOp(op, Box::new(expr))
    }
    pub fn new_subscription(array_expr: Expr, subscript_expr: Expr) -> Expr {
        Expr::Subscription(Box::new(array_expr), Box::new(subscript_expr))
    }
}

#[derive(Copy, Clone)]
pub enum BinaryOpCode {
    Mul,
    Div,
    Add,
    Sub,
    Mod,

    BoolOr,
    BoolAnd,

    LessThan,
    MoreThan,
    LessThanOrEqual,
    MoreThanOrEqual,
    Equal,
    NotEqual,

    BitOr,
    BitXor,
    BitAnd,
    LeftShift,
    RightShift,
}

#[derive(Copy, Clone)]
pub enum UnaryOpCode {
    Neg,
    Plus,
    BitNot,
    BoolNot,
}

pub fn extract_string_literal(token: &str) -> String {
    let token_length = token.len();
    return token.chars().skip(1).take(token_length - 2).collect()
}

fn write_id_list(fmt: &mut Formatter, ids: &Vec<String>) -> Result<(), Error> {
    use std::fmt::Write;
    fmt.write_char('[')?;
    let mut id_iter = ids.iter();
    match id_iter.next() {
        Some(ref s) => {
            fmt.write_str(s)?;
            for id in id_iter {
                fmt.write_str(", ")?;
                fmt.write_str(id)?;
            }
        },
        None => (),
    }
    fmt.write_char(']')
}

impl Debug for Function {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        //write!(fmt, "Function(name: {}, arguments: {:?}, stmts: {:?})", self.name, self.arguments, self.stmts)
        write!(fmt, "Function(name: {}, arguments: ", self.name)?;
        write_id_list(fmt, &self.arguments)?;
        write!(fmt, ", stmts: {:?})", self.stmts)
    }
}

impl Debug for Statement {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Statement::*;
        match *self {
            Return(ref expr) => write!(fmt, "Return({:?})", expr),
            Expr(ref expr) => write!(fmt, "Expr({:?})", expr),
            Assign(ref target, ref expr) => write!(fmt, "Assign(target: {:?}, expr: {:?})", target, expr),
            Declare(ref id, ref expr) => write!(fmt, "Declare(identifier: {}, expr: {:?})", id, expr),
            If(ref test, ref block) => write!(fmt, "If(test: {:?}, block: {:?})", test, block),
            While(ref test, ref block) => write!(fmt, "While(test: {:?}, block: {:?})", test, block)
        }
    }
}

impl Debug for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Expr::*;
        match *self {
            Number(n) => write!(fmt, "{:?}", n),
            Char(c) => write!(fmt, "Char({:?})", c),
            String(ref s) => write!(fmt, "String({:?})", s),
            Array(ref exprs) => write!(fmt, "Array({:?})", exprs),
            BinaryOp(ref l, op, ref r) => write!(fmt, "BinaryOp({:?} {:?} {:?})", l, op, r),
            UnaryOp(op, ref expr) => write!(fmt, "UnaryOp({:?} {:?})", op, expr),
            Call(ref func, ref args) => write!(fmt, "Call(function: {}, arguments: {:?})", func, args),
            Identifier(ref name) => write!(fmt, "Identifier({})", name),
            Subscription(ref array_expr, ref subscript_expr) => {
                write!(fmt, "Subscription(array_expr: {:?}, subscript_expr: {:?})", array_expr, subscript_expr)
            }
            Error => write!(fmt, "error"),
        }
    }
}

impl Debug for BinaryOpCode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::BinaryOpCode::*;
        let op_str = match *self {
            Mul => "*",
            Div => "/",
            Add => "+",
            Sub => "-",
            Mod => "%",

            BoolOr => "or",
            BoolAnd => "and",

            LessThan => "<",
            LessThanOrEqual => "<=",
            MoreThan => ">",
            MoreThanOrEqual => ">=",
            Equal => "==",
            NotEqual => "!=",

            BitOr => "|",
            BitXor => "^",
            BitAnd => "&",
            LeftShift => "<<",
            RightShift => ">>",
        };
        write!(fmt, "{}", op_str)
    }
}

impl Debug for UnaryOpCode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::UnaryOpCode::*;
        let op_str = match *self {
            Neg => "-",
            Plus => "+",
            BitNot => "~",
            BoolNot => "not",
        };
        write!(fmt, "{}", op_str)
    }
}
