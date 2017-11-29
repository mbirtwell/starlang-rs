use std::fmt::{Debug, Formatter, Error};

pub enum Expr {
    Number(i32),
    Char(char),
    String(String),
    Array(Vec<Box<Expr>>),
    BinaryOp(Box<Expr>, BinaryOpCode, Box<Expr>),
    UnaryOp(UnaryOpCode, Box<Expr>),
    Call(String, Vec<Box<Expr>>),
    Identifier(String),
    Subscription(Box<Expr>, Box<Expr>),
    Error,
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
