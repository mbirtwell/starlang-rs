use std::fmt::{Debug, Formatter, Error};

pub enum Expr {
    Number(i32),
    BinaryOp(Box<Expr>, BinaryOpCode, Box<Expr>),
    UnaryOp(UnaryOpCode, Box<Expr>),
    Error,
}

#[derive(Copy, Clone)]
pub enum BinaryOpCode {
    Mul,
    Div,
    Add,
    Sub,

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
    BoolNot,
}

impl Debug for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Expr::*;
        match *self {
            Number(n) => write!(fmt, "{:?}", n),
            BinaryOp(ref l, op, ref r) => write!(fmt, "BinaryOp({:?} {:?} {:?})", l, op, r),
            UnaryOp(op, ref expr) => write!(fmt, "UnaryOp({:?} {:?})", op, expr),
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
            BoolNot => "not",
        };
        write!(fmt, "{}", op_str)
    }
}
