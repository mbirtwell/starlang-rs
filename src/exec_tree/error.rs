use super::super::lexer::Location;
use exec_tree::base::{CodeSite, ExprBox};
use std;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum RuntimeFailureKind {
    ExpectedIntGotArray,
}

impl Display for RuntimeFailureKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            RuntimeFailureKind::ExpectedIntGotArray => write!(f, "Expected int got an array"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ExecError<'a> {
    StaticAnalysisFailed(Vec<StaticAnalysisError<'a>>),
    RuntimeFailure(RuntimeFailureKind, Vec<CodeSite>),
}
pub type ExecResult<'a, T> = std::result::Result<T, ExecError<'a>>;

pub fn runtime_failure(kind: RuntimeFailureKind, expr: &ExprBox) -> ExecError {
    ExecError::RuntimeFailure(
        kind,
        vec![CodeSite {
            start: expr.site.start,
            end: expr.site.end,
        }],
    )
}

#[derive(Debug, PartialEq)]
pub enum StaticAnalysisError<'a> {
    CallUnknownFunction(&'a str, Location, Location),
}

pub type StaticAnalysisErrors<'a> = Vec<StaticAnalysisError<'a>>;
pub type BuildResult<'a, T> = (T, StaticAnalysisErrors<'a>);
