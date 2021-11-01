use std::fmt::{Debug, Error, Formatter};

use lexer::Location;

pub struct Function<'a> {
    pub name: String,
    pub arguments: Vec<&'a str>,
    pub stmts: Vec<Statement<'a>>,
}

impl<'a> Function<'a> {
    pub fn new(name: &str, arguments: Vec<&'a str>, stmts: Vec<Statement<'a>>) -> Function<'a> {
        Function {
            name: name.into(),
            arguments,
            stmts,
        }
    }
}

pub enum Statement<'a> {
    Expr(Expr<'a>),
    Return(Expr<'a>),
    Assign(Expr<'a>, Expr<'a>),
    Declare(&'a str, Expr<'a>),
    If(Expr<'a>, Vec<Statement<'a>>),
    While(Expr<'a>, Vec<Statement<'a>>),
}

pub struct Expr<'a> {
    pub kind: ExprKind<'a>,
    pub start: Location,
    pub end: Location,
}

pub enum ExprKind<'a> {
    Number(i32),
    Char(char),
    String(&'a str),
    Array(Vec<Expr<'a>>),
    BinaryOp(Box<Expr<'a>>, BinaryOpCode, Box<Expr<'a>>),
    UnaryOp(UnaryOpCode, Box<Expr<'a>>),
    Call(&'a str, Vec<Expr<'a>>),
    Identifier(&'a str),
    Subscription(Box<Expr<'a>>, Box<Expr<'a>>),
    Error,
}

macro_rules! cons {
    ( $name:ident ( $( $arg:ident: $typ:ty ), * ) => $kind:ident ) => {
        pub fn $name(start: Location, $( $arg: $typ , )* end: Location) -> Self {
            Self {
                kind: ExprKind::$kind($( $arg.into(), )*),
                start,
                end,
            }
        }
    };
}

impl<'a> Expr<'a> {
    cons! {new_binary_op(lhs:Self, op:BinaryOpCode, rhs:Self ) => BinaryOp}
    cons! {new_unary_op(op: UnaryOpCode, expr: Self) => UnaryOp}
    cons! {new_subscription(array_expr: Self, subscript_expr: Self) => Subscription}
    cons! {new_number(n: i32) => Number}
    cons! {new_string(s: &'a str) => String}
    cons! {new_char(c: char) => Char}
    cons! {new_array(exprs: Vec<Self>) => Array}
    cons! {new_call(func: &'a str, exprs: Vec<Self>) => Call}
    cons! {new_identifier(name: &'a str) => Identifier}
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

fn write_id_list(fmt: &mut Formatter, ids: &[&str]) -> Result<(), Error> {
    use std::fmt::Write;
    fmt.write_char('[')?;
    let mut id_iter = ids.iter();
    if let Some(s) = id_iter.next() {
        fmt.write_str(s)?;
        for id in id_iter {
            fmt.write_str(", ")?;
            fmt.write_str(id)?;
        }
    }
    fmt.write_char(']')
}

impl<'a> Debug for Function<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        //write!(fmt, "Function(name: {}, arguments: {:?}, stmts: {:?})", self.name, self.arguments, self.stmts)
        write!(fmt, "Function(name: {}, arguments: ", self.name)?;
        write_id_list(fmt, &self.arguments)?;
        write!(fmt, ", stmts: {:?})", self.stmts)
    }
}

impl<'a> Debug for Statement<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::Statement::*;
        match *self {
            Return(ref expr) => write!(fmt, "Return({:?})", expr),
            Expr(ref expr) => write!(fmt, "Expr({:?})", expr),
            Assign(ref target, ref expr) => {
                write!(fmt, "Assign(target: {:?}, expr: {:?})", target, expr)
            }
            Declare(ref id, ref expr) => {
                write!(fmt, "Declare(identifier: {}, expr: {:?})", id, expr)
            }
            If(ref test, ref block) => write!(fmt, "If(test: {:?}, block: {:?})", test, block),
            While(ref test, ref block) => {
                write!(fmt, "While(test: {:?}, block: {:?})", test, block)
            }
        }
    }
}

impl<'a> Debug for Expr<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        self.kind.fmt(fmt)
    }
}

impl<'a> Debug for ExprKind<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        use self::ExprKind::*;
        match *self {
            Number(n) => write!(fmt, "{:?}", n),
            Char(c) => write!(fmt, "Char({:?})", c),
            String(ref s) => write!(fmt, "String({:?})", s),
            Array(ref exprs) => write!(fmt, "Array({:?})", exprs),
            BinaryOp(ref l, op, ref r) => write!(fmt, "BinaryOp({:?} {:?} {:?})", l, op, r),
            UnaryOp(op, ref expr) => write!(fmt, "UnaryOp({:?} {:?})", op, expr),
            Call(ref func, ref args) => {
                write!(fmt, "Call(function: {}, arguments: {:?})", func, args)
            }
            Identifier(ref name) => write!(fmt, "Identifier({})", name),
            Subscription(ref array_expr, ref subscript_expr) => write!(
                fmt,
                "Subscription(array_expr: {:?}, subscript_expr: {:?})",
                array_expr, subscript_expr
            ),
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
