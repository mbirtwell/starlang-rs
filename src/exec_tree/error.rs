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
pub enum ExecError {
    StaticAnalysisFailed(Vec<StaticAnalysisError>),
    RuntimeFailure(RuntimeFailureKind, Vec<CodeSite>),
}
pub type ExecResult<T> = std::result::Result<T, ExecError>;

pub fn runtime_failure(kind: RuntimeFailureKind, expr: &ExprBox) -> ExecError {
    ExecError::RuntimeFailure(kind, vec![expr.site])
}

#[derive(Debug, PartialEq)]
pub enum StaticAnalysisError {
    CallUnknownFunction(String, Location, Location),
}

pub type StaticAnalysisErrors = Vec<StaticAnalysisError>;
pub type BuildResult<'a, T> = (T, StaticAnalysisErrors);
